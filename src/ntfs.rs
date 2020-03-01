#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::io::Error;

use super::volume::Volume;
use winapi::shared::minwindef::{DWORD, LPDWORD, LPVOID, WORD};
use winapi::shared::ntdef::{DWORDLONG, USN, WCHAR};
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::winioctl::{
    FSCTL_CREATE_USN_JOURNAL, FSCTL_ENUM_USN_DATA, FSCTL_QUERY_USN_JOURNAL,
};
use winapi::um::winnt::LARGE_INTEGER;

#[derive(Debug, Default)]
pub struct USN_JOURNAL_DATA {
    pub UsnJournalID: DWORDLONG,
    pub FirstUsn: USN,
    pub NextUsn: USN,
    pub LowestValidUsn: USN,
    pub MaxUsn: USN,
    pub MaximumSize: DWORDLONG,
    pub AllocationDelta: DWORDLONG,
}

#[derive(Debug, Default)]
pub struct CREATE_USN_JOURNAL_DATA {
    MaximumSize: DWORDLONG,
    AllocationDelta: DWORDLONG,
}

#[derive(Debug, Default)]
pub struct MFT_ENUM_DATA {
    StartFileReferenceNumber: DWORDLONG,
    LowUsn: USN,
    HighUsn: USN,
}

struct LargeInteger(LARGE_INTEGER);

impl std::fmt::Debug for LargeInteger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LARGE_INTEGER {}", unsafe { self.0.QuadPart() })
    }
}

impl std::default::Default for LargeInteger {
    fn default() -> Self {
        LargeInteger(unsafe { std::mem::zeroed::<LARGE_INTEGER>() })
    }
}

#[derive(Default, Debug)]
pub struct USN_RECORD {
    RecordLength: DWORD,
    MajorVersion: WORD,
    MinorVersion: WORD,
    FileReferenceNumber: DWORDLONG,
    ParentFileReferenceNumber: DWORDLONG,
    Usn: USN,
    TimeStamp: LargeInteger,
    Reason: DWORD,
    SourceInfo: DWORD,
    SecurityId: DWORD,
    FileAttributes: DWORD,
    FileNameLength: WORD,
    FileNameOffset: WORD,
    FileName: [WCHAR; 32],
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
                std::ptr::null_mut(),
                0,
                &mut returned_bytes as LPDWORD,
                std::ptr::null_mut(),
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
                std::ptr::null_mut(),
                0,
                &mut usn_journal_data as *mut USN_JOURNAL_DATA as LPVOID,
                std::mem::size_of_val(&usn_journal_data) as DWORD,
                &mut returned_bytes as LPDWORD,
                std::ptr::null_mut(),
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
        let mut usn_record: USN_RECORD = Default::default();
        let res = unsafe {
            DeviceIoControl(
                self.handle,
                FSCTL_ENUM_USN_DATA,
                &mft_enum_data as *const MFT_ENUM_DATA as LPVOID,
                std::mem::size_of_val(&mft_enum_data) as DWORD,
                &mut usn_record as *mut USN_RECORD as LPVOID,
                std::mem::size_of_val(&usn_record) as DWORD,
                &mut returned_bytes as LPDWORD,
                std::ptr::null_mut(),
            )
        };
        match res {
            0 => Err(Error::last_os_error()),
            _ => Ok(usn_record),
        }
    }
}
