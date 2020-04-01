use ddup::utils::{hash_map_to_paths, usn_records_to_hash_map};
use ddup::Ntfs;
use ddup::Volume;
use ddup::UsnRange;

fn doit() {
    let volume = Volume::open(r"\\.\C:").unwrap();

    let journal = volume.query_usn_journal().unwrap();

    let range = UsnRange {
        low: journal.LowestValidUsn,
        high: journal.NextUsn,
    };

    let usn_records = volume.iterate_usn_records(&range);
    let map = usn_records_to_hash_map(usn_records);
    let paths = hash_map_to_paths(&map);
    for (i, path) in paths.iter().enumerate() {
        println!("{:<8} {}", i, path.to_str().unwrap());
    }
}

fn main() {
    doit()
}
