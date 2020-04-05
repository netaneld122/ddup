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
    pub fn new(drive: &str) -> Result<Self, Error> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use walkdir;
    use std::time::Instant;
    use std::collections::HashSet;

    #[test]
    fn compare_walkdir_to_dirlist() {
        println!("What is this\r\n");
        let instant = Instant::now();
        let mut v1 = Vec::new();
        for p in walkdir::WalkDir::new(r"G:\") {
            if let Ok(d) = p {
                if d.file_type().is_file() {
                    v1.push(String::from(d.path().to_str().unwrap()));
                }
            }
        }
        println!("WalkDir got {} entries in {} seconds", v1.len(), instant.elapsed().as_secs_f32());

        let instant = Instant::now();
        let mut v2 = Vec::new();
        let dirlist = DirList::new("G:").unwrap();
        for p in dirlist.iter() {
            v2.push(String::from(p.to_str().unwrap()));
        }
        println!("Dirlist got {} entries in {} seconds", v2.len(), instant.elapsed().as_secs_f32());

        let set1: HashSet<String> = v1.iter().cloned().map(|s|s.to_lowercase()).collect();
        let set2: HashSet<String> = v2.iter().cloned().map(|s|s.to_lowercase()).collect();

        println!("a - b:");
        for diff in set1.difference(&set2).into_iter().take(100) {
            println!("\t{}", diff);
        }

        println!("b - a:");
        for diff in set2.difference(&set1).into_iter().take(10) {
            println!("\t{}", diff);
        }
    }
}