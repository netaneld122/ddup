#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use winapi::shared::minwindef::{DWORD, WORD};
use winapi::shared::ntdef::{DWORDLONG, USN, WCHAR};
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