use ddup::Ntfs;
use ddup::Volume;
use ddup::UsnRange;

#[test]
fn enumerate_mft_records() -> Result<(), std::io::Error> {
    let volume = Volume::open(r"\\.\C:")?;

    let journal = volume.query_usn_journal()?;

    let range = UsnRange {
        low: journal.LowestValidUsn,
        high: journal.NextUsn,
    };

    for record in volume.usn_records(&range).take(1000) {
        println!("{:x} {:x} {:?} `{}`",
                 record.id,
                 record.parent_id,
                 record.record_type,
                 record.filename);
    }

    Ok(())
}

#[test]
fn create_journal() -> Result<(), std::io::Error> {
    let volume = Volume::open(r"\\.\C:")?;

    volume.create_usn_journal()?;

    Ok(())
}