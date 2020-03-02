use ddup::Ntfs;
use ddup::Volume;

#[test]
fn enumerate_one_mft_record() -> Result<(), std::io::Error> {
    let volume = Volume::open(r"\\.\C:")?;
    volume.create_usn_journal()?;
    let journal = volume.query_usn_journal()?;
    let record = volume.enum_usn_data(journal.LowestValidUsn, journal.NextUsn)?;

    let file_name = String::from_utf16_lossy(&{ record.FileName });
    println!("{:?}", file_name);

    Ok(())
}
