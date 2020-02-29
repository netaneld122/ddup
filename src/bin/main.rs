use ddup::ntfs;
use std::ffi::CString;

fn main() {
    let path = CString::new(r"\\.\C:").unwrap();
    let handle = ntfs::open_volume(&path).unwrap();

    ntfs::create_usn_journal(handle).unwrap();
    let journal = ntfs::query_usn_journal(handle).unwrap();

    println!("Journal {:#?}", journal);
}
