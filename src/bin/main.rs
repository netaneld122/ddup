use ddup::ntfs::Ntfs;
use ddup::volume::Volume;

fn main() {
    let volume = Volume::open(r"\\.\C:").unwrap();
    volume.create_usn_journal().unwrap();
    let journal = volume.query_usn_journal().unwrap();
    println!("Journal {:#?}", journal);
    let data = volume.enum_usn_data(journal.LowestValidUsn, journal.NextUsn).unwrap();
    println!("Journal {:#?}", data);
}
