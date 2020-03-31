use ddup::Ntfs;
use ddup::Volume;

#[test]
fn enumerate_mft_records() -> Result<(), std::io::Error> {
    let volume = Volume::open(r"\\.\C:")?;
    volume.create_usn_journal()?;
    let journal = volume.query_usn_journal()?;
    println!("{:#?}", journal);

    let data = volume.enum_usn_data(journal.LowestValidUsn, journal.NextUsn)?;
    for record in data {
        println!("{}", record.filename);
    }

    Ok(())
}
