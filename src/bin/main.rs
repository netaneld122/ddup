use ddup::DirList;

fn main() {
    let dirlist = DirList::new("C:").unwrap();
    for (i, path) in dirlist.iter().enumerate() {
        println!("{:<8} {}", i, path.to_str().unwrap());
    }
}