#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::io::Error;
use std::ptr::null_mut;

use super::volume::Volume;

use winapi::shared::minwindef::{DWORD, LPDWORD, LPVOID, WORD};
use winapi::shared::ntdef::{DWORDLONG, USN, WCHAR};
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::winioctl::{
    FSCTL_CREATE_USN_JOURNAL, FSCTL_ENUM_USN_DATA, FSCTL_QUERY_USN_JOURNAL,
};
use winapi::um::winnt::LARGE_INTEGER;

#[repr(C)]
#[derive(Default, Debug)]
pub struct USN_JOURNAL_DATA {
    pub UsnJournalID: DWORDLONG,
    pub FirstUsn: USN,
    pub NextUsn: USN,
    pub LowestValidUsn: USN,
    pub MaxUsn: USN,
    pub MaximumSize: DWORDLONG,
    pub AllocationDelta: DWORDLONG,
}

#[repr(C)]
#[derive(Default)]
pub struct CREATE_USN_JOURNAL_DATA {
    pub MaximumSize: DWORDLONG,
    pub AllocationDelta: DWORDLONG,
}

#[repr(C)]
#[derive(Default)]
pub struct MFT_ENUM_DATA {
    pub StartFileReferenceNumber: DWORDLONG,
    pub LowUsn: USN,
    pub HighUsn: USN,
}

#[repr(C)]
pub struct USN_RECORD {
    pub RecordLength: DWORD,
    pub MajorVersion: WORD,
    pub MinorVersion: WORD,
    pub FileReferenceNumber: DWORDLONG,
    pub ParentFileReferenceNumber: DWORDLONG,
    pub Usn: USN,
    pub TimeStamp: LARGE_INTEGER,
    pub Reason: DWORD,
    pub SourceInfo: DWORD,
    pub SecurityId: DWORD,
    pub FileAttributes: DWORD,
    pub FileNameLength: WORD,
    pub FileNameOffset: WORD,
    pub FileName: [WCHAR; 1],
}

pub struct UsnRecord {
    pub filename: String,
}

pub struct UsnData {
    buffer: [u8; 0x10000],
    size: usize,
    offset: usize,
}

impl Iterator for UsnData {
    type Item = UsnRecord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.size {
            return None;
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

        Some(UsnRecord { filename })
    }
}

pub trait Ntfs {
    fn create_usn_journal(&self) -> Result<(), Error>;
    fn query_usn_journal(&self) -> Result<USN_JOURNAL_DATA, Error>;
    fn enum_usn_data(&self, low: USN, high: USN) -> Result<UsnData, Error>;
}

impl Ntfs for Volume {
    fn create_usn_journal(&self) -> Result<(), Error> {
        let mut returned_bytes: u32 = 0;
        let input = CREATE_USN_JOURNAL_DATA {
            MaximumSize: 1024 * 1024 * 100,
            AllocationDelta: 1024,
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

    fn enum_usn_data(&self, low: USN, high: USN) -> Result<UsnData, Error> {
        let mut returned_bytes: u32 = 0;
        let mft_enum_data = MFT_ENUM_DATA {
            StartFileReferenceNumber: 0,
            LowUsn: low,
            HighUsn: high,
        };
        let mut buffer = [0u8; 0x10000];

        let res = unsafe {
            DeviceIoControl(
                self.handle,
                FSCTL_ENUM_USN_DATA,
                &mft_enum_data as *const MFT_ENUM_DATA as LPVOID,
                std::mem::size_of_val(&mft_enum_data) as DWORD,
                buffer.as_mut_ptr() as *mut USN_RECORD as LPVOID,
                buffer.len() as DWORD,
                &mut returned_bytes as LPDWORD,
                null_mut(),
            )
        };

        if res == 0 {
            return Err(Error::last_os_error());
        }

        match res {
            0 => Err(Error::last_os_error()),
            _ => Ok(UsnData {
                buffer,
                offset: std::mem::size_of::<USN>(), // Skip USN header
                size: returned_bytes as usize,
            }),
        }
    }
}
