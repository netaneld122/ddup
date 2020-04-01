use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{UsnRecord, UsnRecordType};

pub fn usn_records_to_hash_map(iterator: impl Iterator<Item=UsnRecord>)
                               -> HashMap<u64, UsnRecord> {
    iterator.map(|record| (record.id, record)).collect()
}

pub fn hash_map_to_paths(map: &HashMap<u64, UsnRecord>) -> Vec<PathBuf> {
    let mut full_paths = Vec::new();

    for record in map.values() {
        if let UsnRecordType::Directory = record.record_type {
            continue;
        }

        let mut path= PathBuf::from(&record.filename);

        // Traverse hashmap to find parents
        let mut current = record;
        while let Some(parent) = map.get(&current.parent_id) {
            current = parent;
            // Prepend parent filename
            path = Path::new(&current.filename).join(&path);
        }

        full_paths.push(path);
    }

    full_paths
}