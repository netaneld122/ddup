use std::path::{PathBuf, Path};
use std::io::Error;

use super::utils::{hash_map_to_paths, usn_records_to_hash_map};
use super::Ntfs;
use super::Volume;
use super::UsnRange;

pub struct DirList {
    paths: Vec<PathBuf>,
}

impl DirList {
    pub fn new(drive: &str) -> Result<DirList, Error> {
        let volume = Volume::open(&(String::from(r"\\.\") + drive))?;

        let journal = volume.query_usn_journal()?;

        let range = UsnRange {
            low: journal.LowestValidUsn,
            high: journal.NextUsn,
        };

        let usn_records = volume.usn_records(&range);
        let map = usn_records_to_hash_map(usn_records);
        let paths = hash_map_to_paths(&map);

        // Prepend the drive
        let paths = paths.iter()
            .map(|p| Path::new(drive).join(r"\").join(&p))
            .collect();

        Ok(DirList { paths })
    }

    pub fn iter(&self) -> impl Iterator<Item=&PathBuf> {
        self.paths.iter()
    }
}