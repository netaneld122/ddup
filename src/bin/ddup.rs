use std::time::Instant;
use std::path::PathBuf;

use clap::{Arg, App, ArgMatches};

use ddup::algorithm;

fn parse_args() -> ArgMatches<'static> {
    App::new("ddup")
        .about("This tool identifies duplicated files in Windows NTFS Volumes")
        .arg(Arg::with_name("drive")
            .short("d")
            .long("drive")
            .value_name("DRIVE")
            .help("Configure the drive letter to scan (example C:)")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("postfix")
            .long("postfix")
            .value_name("POSTFIX")
            .help("Scan only paths that end with POSTFIX (example .dmp)")
            .takes_value(true))
        .arg(Arg::with_name("prefix")
            .long("prefix")
            .value_name("PREFIX")
            .help(r"Scan only paths that start with PREFIX (example C:\Windows)")
            .takes_value(true))
        .get_matches()
}

fn main() {
    let args = parse_args();
    let drive = args.value_of("drive").unwrap();

    let i = Instant::now();

    if let Some(postfix) = args.value_of("postfix") {
        println!("Scanning drive {} with postfix {}", drive, postfix);

        algorithm::run(drive, |path: &&PathBuf| path
            .to_string_lossy()
            .to_lowercase()
            .ends_with(&postfix.to_lowercase()));
    } else if let Some(prefix) = args.value_of("prefix") {
        println!("Scanning drive {} with prefix {}", drive, prefix);

        algorithm::run(drive, |path: &&PathBuf| path
            .to_string_lossy()
            .to_lowercase()
            .starts_with(&prefix.to_lowercase()));
    } else {
        println!("Scanning drive {}", drive);

        algorithm::run(drive, |_| true);
    }

    println!("Finished in {} seconds", i.elapsed().as_secs_f32());
}