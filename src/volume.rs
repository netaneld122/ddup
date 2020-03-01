use std::ffi::CString;
use std::io::Error;

use winapi::um::fileapi::CreateFileA;
use winapi::um::fileapi::OPEN_EXISTING;
use winapi::um::handleapi::CloseHandle;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, HANDLE};

pub struct Volume {
    pub handle: HANDLE,
}

impl Drop for Volume {
    fn drop(&mut self) {
        if self.handle != INVALID_HANDLE_VALUE {
            unsafe { CloseHandle(self.handle) };
        }
    }
}

impl Volume {
    pub fn open(name: &str) -> Result<Volume, Error> {
        let name = CString::new(name).unwrap();
        let handle = unsafe {
            CreateFileA(
                name.as_ptr(),
                GENERIC_READ,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null_mut(),
                OPEN_EXISTING,
                0,
                std::ptr::null_mut(),
            )
        };

        match handle {
            INVALID_HANDLE_VALUE => Err(Error::last_os_error()),
            _ => Ok(Volume { handle }),
        }
    }
}
