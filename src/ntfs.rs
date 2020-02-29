#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CStr;
use std::io::Error;

use winapi;
use winapi::shared::minwindef::{DWORD, LPDWORD, LPVOID};
use winapi::shared::ntdef::{DWORDLONG, USN};
use winapi::um::fileapi::OPEN_EXISTING;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::winioctl::{FSCTL_CREATE_USN_JOURNAL, FSCTL_QUERY_USN_JOURNAL};
use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE, HANDLE};

#[derive(Debug, Default)]
pub struct USN_JOURNAL_DATA {
    UsnJournalID: DWORDLONG,
    FirstUsn: USN,
    NextUsn: USN,
    LowestValidUsn: USN,
    MaxUsn: USN,
    MaximumSize: DWORDLONG,
    AllocationDelta: DWORDLONG,
}

#[derive(Debug, Default)]
pub struct CREATE_USN_JOURNAL_DATA {
    MaximumSize: DWORDLONG,
    AllocationDelta: DWORDLONG,
}

pub fn open_volume(name: &CStr) -> Result<HANDLE, Error> {
    let handle = unsafe {
        winapi::um::fileapi::CreateFileA(
            name.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        )
    };

    match handle {
        INVALID_HANDLE_VALUE => Err(Error::last_os_error()),
        _ => Ok(handle),
    }
}

pub fn create_usn_journal(handle: HANDLE) -> Result<(), Error> {
    let mut returned_bytes: u32 = 0;
    let input = CREATE_USN_JOURNAL_DATA {
        MaximumSize: 1024 * 1024 * 100,
        AllocationDelta: 1024,
    };
    let res = unsafe {
        winapi::um::ioapiset::DeviceIoControl(
            handle,
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

pub fn query_usn_journal(handle: HANDLE) -> Result<USN_JOURNAL_DATA, Error> {
    let mut returned_bytes: u32 = 0;
    let mut usn_journal_data: USN_JOURNAL_DATA = Default::default();
    let res = unsafe {
        winapi::um::ioapiset::DeviceIoControl(
            handle,
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
