use ddup::Ntfs;
use ddup::Volume;
use ddup::UsnRange;

#[test]
fn enumerate_mft_records() -> Result<(), std::io::Error> {
    let volume = Volume::open(r"\\.\C:")?;

    let journal = volume.query_usn_journal()?;

    let mut iterations = 0;
    let range = UsnRange {
        low: journal.LowestValidUsn,
        high: journal.NextUsn };

    for record in volume.iterate_usn_records(&range) {
        println!("{}", record.filename);

        // Stop after a certain amount of iterations
        iterations = iterations + 1;
        if iterations > 1000 {
            return Ok(());
        };
    }

    Ok(())
}

#[test]
fn create_journal() -> Result<(), std::io::Error> {
    let volume = Volume::open(r"\\.\C:")?;

    volume.create_usn_journal()?;

    Ok(())
}
