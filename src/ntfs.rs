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

#[repr(packed)]
#[derive(Default)]
pub struct USN_JOURNAL_DATA {
    pub UsnJournalID: DWORDLONG,
    pub FirstUsn: USN,
    pub NextUsn: USN,
    pub LowestValidUsn: USN,
    pub MaxUsn: USN,
    pub MaximumSize: DWORDLONG,
    pub AllocationDelta: DWORDLONG,
}

#[repr(packed)]
#[derive(Default)]
pub struct CREATE_USN_JOURNAL_DATA {
    pub MaximumSize: DWORDLONG,
    pub AllocationDelta: DWORDLONG,
}

#[repr(packed)]
#[derive(Default)]
pub struct MFT_ENUM_DATA {
    pub StartFileReferenceNumber: DWORDLONG,
    pub LowUsn: USN,
    pub HighUsn: USN,
}

#[repr(packed)]
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
    pub FileName: [WCHAR; 16],
}

pub trait Ntfs {
    fn create_usn_journal(&self) -> Result<(), Error>;
    fn query_usn_journal(&self) -> Result<USN_JOURNAL_DATA, Error>;
    fn enum_usn_data(&self, low: USN, high: USN) -> Result<USN_RECORD, Error>;
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

    fn enum_usn_data(&self, low: USN, high: USN) -> Result<USN_RECORD, Error> {
        let mut returned_bytes: u32 = 0;
        let mft_enum_data = MFT_ENUM_DATA {
            StartFileReferenceNumber: 0,
            LowUsn: low,
            HighUsn: high,
        };
        let mut records = [0u8; std::mem::size_of::<USN>() + 0x10000];

        let res = unsafe {
            DeviceIoControl(
                self.handle,
                FSCTL_ENUM_USN_DATA,
                &mft_enum_data as *const MFT_ENUM_DATA as LPVOID,
                std::mem::size_of_val(&mft_enum_data) as DWORD,
                records.as_mut_ptr() as *mut USN_RECORD as LPVOID,
                records.len() as DWORD,
                &mut returned_bytes as LPDWORD,
                null_mut(),
            )
        };

        // Skip USN
        let records = &records[std::mem::size_of::<USN>()..];
        let usn_record: USN_RECORD = unsafe { std::mem::transmute_copy(&records.as_ptr()) };

        match res {
            0 => Err(Error::last_os_error()),
            _ => Ok(usn_record),
        }
    }
}
