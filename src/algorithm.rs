use std::fs;
use std::io::{self, Seek, Read};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::cmp::min;
use std::ops::FnMut;

use crc::{crc32, Hasher32};

use super::DirList;

#[derive(Debug)]
pub enum Comparison {
    Fuzzy,
    Strict,
}

fn calculate_fuzzy_hash(size: u64, file: &mut fs::File) -> u32 {
    let mut digest = crc32::Digest::new(crc32::IEEE);
    let mut buffer = [0u8; 1024 * 4];
    let mut offset: u64 = 0;

    // Digest with exponentially decreasing density
    while offset + (buffer.len() as u64) < size {
        file.seek(io::SeekFrom::Start(offset)).unwrap();
        let bytes_read = file.read(&mut buffer).unwrap() as u64;
        digest.write(&mut buffer[..bytes_read as usize]);
        offset += bytes_read;
        offset *= 2;
    }

    // Digest the last chunk
    let offset_from_end = -min(size as i64, buffer.len() as i64);
    file.seek(io::SeekFrom::End(offset_from_end)).unwrap();
    file.read(&mut buffer).unwrap();
    digest.write(&mut buffer);

    digest.sum32()
}

// @TODO: Replace this with sha512
fn calculate_hash(file: &mut fs::File) -> u32 {
    let mut digest = crc32::Digest::new(crc32::IEEE);
    let mut buffer = [0u8; 1024 * 4];

    let mut bytes_read = file.read(&mut buffer).unwrap();
    while bytes_read > 0 {
        digest.write(&mut buffer[..bytes_read]);
        bytes_read = file.read(&mut buffer).unwrap();
    }

    digest.sum32()
}

fn reduce_by_content<'a>(size: u64, paths: &[&'a Path], comparison: &Comparison) -> Vec<Vec<&'a Path>> {
    let mut map = HashMap::with_capacity(paths.len());

    // Group files by crc32 of beginning
    for path in paths {
        let mut file = match fs::File::open(path) {
            Ok(f) => f,
            _ => continue,
        };

        let hash = match comparison {
            Comparison::Fuzzy => calculate_fuzzy_hash(size, &mut file),
            Comparison::Strict => calculate_hash(&mut file),
        };
        map.entry(hash)
            .or_insert(Vec::new())
            .push(*path);
    }

    // Filter out single occurrences
    map = map.into_iter()
        .filter(|(_, v)| v.len() > 1)
        .collect();

    map.values().cloned().collect()
}

pub fn run<P>(drive: &str, filter: P, comparison: &Comparison)
    where P: FnMut(&&PathBuf) -> bool {

    println!("Generating recursive dirlist");
    let dirlist = DirList::new(drive).unwrap();

    println!("Grouping by file size");
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

    println!("Grouping by hash");
    // Print all duplicates
    let mut i = 0;
    for (size, same_size_paths) in map.into_iter() {
        for same_crc_paths in reduce_by_content(size, &same_size_paths, comparison).into_iter() {
            println!("Potential duplicates [{} bytes]", size);
            for path in same_crc_paths {
                i += 1;
                println!("\t{} {}", i, path.to_str().unwrap());
            }
        }
    }
}