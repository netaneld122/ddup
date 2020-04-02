use std::fs;
use std::io::{self, Seek, Read};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::cmp::min;
use std::ops::FnMut;

use crc::{crc32, Hasher32};

use super::DirList;

fn reduce_by_crc<'a>(size: u64, paths: &[&'a Path]) -> Vec<Vec<&'a Path>> {
    let mut map = HashMap::with_capacity(paths.len());

    // Group files by crc32 of beginning
    for path in paths {
        let mut file = match fs::File::open(path) {
            Ok(f) => f,
            _ => continue,
        };

        let mut digest = crc32::Digest::new(crc32::IEEE);
        let mut buffer = [0u8; 1024 * 4];

        // Seek to beginning
        file.seek(io::SeekFrom::Start(0)).unwrap();
        file.read(&mut buffer).unwrap();
        digest.write(&mut buffer);

        if size > buffer.len() as u64 * 2 {
            // Seek to the middle
            file.seek(io::SeekFrom::Start(size / 2 - buffer.len() as u64 / 2)).unwrap();
            file.read(&mut buffer).unwrap();
            digest.write(&mut buffer);

            // Seek to the end
            let offset_from_end = min(size as i64, buffer.len() as i64);
            file.seek(io::SeekFrom::End(offset_from_end)).unwrap();
            file.read(&mut buffer).unwrap();
            digest.write(&mut buffer);
        }

        map.entry(digest.sum32())
            .or_insert(Vec::new())
            .push(*path);
    }

    // Filter out single occurrences
    map = map.into_iter()
        .filter(|(_, v)| v.len() > 1)
        .collect();

    map.values().cloned().collect()
}

pub fn run<P>(drive: &str, filter: P)
    where P: FnMut(&&PathBuf) -> bool {
    let dirlist = DirList::new(drive).unwrap();

    // Group files by size
    let mut map: HashMap<u64, Vec<&Path>> = HashMap::new();

    for path in dirlist.iter().filter(filter) {
        let file_size = match fs::metadata(path) {
            Ok(m) => m.len(),
            _ => continue
        };

        map.entry(file_size)
            .or_insert(Vec::new())
            .push(path);
    }

    // Filter out single occurrences
    map = map.into_iter()
        .filter(|(_, v)| v.len() > 1)
        .collect();

    // Print all duplicates
    let mut i = 0;
    for (size, same_size_paths) in map.into_iter() {
        for same_crc_paths in reduce_by_crc(size, &same_size_paths).into_iter() {
            println!("Duplicated files [{} bytes]", size);
            for path in same_crc_paths {
                i += 1;
                println!("\t{} {}", i, path.to_str().unwrap());
            }
        }
    }
}