use std::io::Error;
use std::ptr::null_mut;

use super::volume::Volume;
use super::winioctl::{USN_JOURNAL_DATA, CREATE_USN_JOURNAL_DATA, MFT_ENUM_DATA, USN_RECORD};

use winapi::shared::minwindef::{DWORD, LPDWORD, LPVOID};
use winapi::shared::ntdef::USN;
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::winioctl::{
    FSCTL_CREATE_USN_JOURNAL, FSCTL_ENUM_USN_DATA, FSCTL_QUERY_USN_JOURNAL,
};
use winapi::um::winnt::FILE_ATTRIBUTE_DIRECTORY;

#[derive(Debug)]
pub enum UsnRecordType {
    File,
    Directory,
}

pub struct UsnRecord {
    pub id: u64,
    pub parent_id: u64,
    pub record_type: UsnRecordType,
    pub filename: String,
}

pub struct UsnRange {
    pub low: USN,
    pub high: USN,
}

const BUFFER_SIZE: usize = 1024 * 64;

pub struct UsnRecordsIterator<'a> {
    volume: &'a Volume,
    buffer: [u8; BUFFER_SIZE],
    reference_number: u64,
    usn_range: &'a UsnRange,
    size: usize,
    offset: usize,
}

impl<'a> UsnRecordsIterator<'a> {
    fn new(volume: &'a Volume, usn_range: &'a UsnRange) -> UsnRecordsIterator<'a> {
        UsnRecordsIterator {
            volume,
            buffer: [0; BUFFER_SIZE],
            reference_number: 0,
            usn_range,
            size: 0,
            offset: 0,
        }
    }
}

impl<'a> UsnRecordsIterator<'a> {
    fn fetch(&mut self) -> Result<(), Error> {
        let mut returned_bytes: u32 = 0;
        let mft_enum_data = MFT_ENUM_DATA {
            StartFileReferenceNumber: self.reference_number,
            LowUsn: self.usn_range.low,
            HighUsn: self.usn_range.high,
        };

        let res = unsafe {
            DeviceIoControl(
                self.volume.handle,
                FSCTL_ENUM_USN_DATA,
                &mft_enum_data as *const MFT_ENUM_DATA as LPVOID,
                std::mem::size_of_val(&mft_enum_data) as DWORD,
                self.buffer.as_mut_ptr() as *mut USN_RECORD as LPVOID,
                self.buffer.len() as DWORD,
                &mut returned_bytes as LPDWORD,
                null_mut(),
            )
        };

        if res != 0 {
            self.reference_number = unsafe {
                *(self.buffer.as_ptr() as *const u64)
            };

            self.size = returned_bytes as usize;
            self.offset = std::mem::size_of_val(&self.reference_number);
        }

        match res {
            0 => Err(Error::last_os_error()),
            _ => Ok(()),
        }
    }
}

impl<'a> Iterator for UsnRecordsIterator<'a> {
    type Item = UsnRecord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.size {
            match self.fetch() {
                Err(err) if Some(38) == err.raw_os_error() => {
                    // EOF
                    return None;
                }
                Err(err) => { panic!("Usn records iteration failed with {}", err); }
                _ => ()
            }
        }

        let base = self.buffer.as_ptr();
        let ptr = unsafe { base.offset(self.offset as isize) };
        let usn_record: &USN_RECORD = unsafe { std::mem::transmute(ptr) };

        let filename = unsafe {
            ptr.offset(usn_record.FileNameOffset as isize) as *const u16
        };
        let filename = unsafe {
            std::slice::from_raw_parts(
                filename,
                (usn_record.FileNameLength / 2) as usize)
        };
        let filename = String::from_utf16_lossy(filename);

        // Advance to next record
        self.offset += usn_record.RecordLength as usize;

        let record_type =
            if usn_record.FileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
                UsnRecordType::Directory
            } else {
                UsnRecordType::File
            };

        Some(UsnRecord {
            id: usn_record.FileReferenceNumber,
            parent_id: usn_record.ParentFileReferenceNumber,
            record_type,
            filename,
        })
    }
}

pub trait Ntfs {
    fn create_usn_journal(&self) -> Result<(), Error>;
    fn query_usn_journal(&self) -> Result<USN_JOURNAL_DATA, Error>;
    fn usn_records<'a>(&'a self, range: &'a UsnRange) -> UsnRecordsIterator<'a>;
}

impl Ntfs for Volume {
    fn create_usn_journal(&self) -> Result<(), Error> {
        let mut returned_bytes: u32 = 0;
        let input = CREATE_USN_JOURNAL_DATA {
            MaximumSize: 1024 * 1024 * 100,
            AllocationDelta: 1024 * 1024,
        };

        let res = unsafe {
            DeviceIoControl(
                self.handle,
                FSCTL_CREATE_USN_JOURNAL,
                &input as *const CREATE_USN_JOURNAL_DATA as LPVOID,
                std::mem::size_of_val(&input) as DWORD,
                null_mut(),
                0,
                &mut returned_bytes as LPDWORD,
                null_mut(),
            )
        };

        match res {
            0 => Err(Error::last_os_error()),
            _ => Ok(()),
        }
    }

    fn query_usn_journal(&self) -> Result<USN_JOURNAL_DATA, Error> {
        let mut returned_bytes: u32 = 0;
        let mut usn_journal_data: USN_JOURNAL_DATA = Default::default();

        let res = unsafe {
            DeviceIoControl(
                self.handle,
                FSCTL_QUERY_USN_JOURNAL,
                null_mut(),
                0,
                &mut usn_journal_data as *mut USN_JOURNAL_DATA as LPVOID,
                std::mem::size_of_val(&usn_journal_data) as DWORD,
                &mut returned_bytes as LPDWORD,
                null_mut(),
            )
        };

        match res {
            0 => Err(Error::last_os_error()),
            _ => Ok(usn_journal_data),
        }
    }

    fn usn_records<'a>(&'a self, usn_range: &'a UsnRange) -> UsnRecordsIterator<'a> {
        UsnRecordsIterator::new(&self, usn_range)
    }
}