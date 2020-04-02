use std::time::Instant;
use std::path::PathBuf;

use clap::{Arg, App, ArgMatches};

use glob::{Pattern, MatchOptions};

use ddup::algorithm;

fn parse_args() -> ArgMatches<'static> {
    App::new("ddup")
        .about("This tool identifies duplicated files in Windows NTFS Volumes")
        .arg(Arg::with_name("drive")
            .help("The drive letter to scan (example `C:`)")
            .required(true)
            .index(1))
        .arg(Arg::with_name("match")
            .short("m")
            .long("match")
            .value_name("PATTERN")
            .help("Scan only paths that match the glob pattern (example `**.dmp`)")
            .takes_value(true))
        .arg(Arg::with_name("i")
            .short("i")
            .help("Treat the matcher as case-insensitive"))
        .get_matches()
}

fn main() {
    let args = parse_args();

    let drive = args.value_of("drive")
        .expect("Drive format is `<letter>:`");

    let instant = Instant::now();

    if let Some(pattern) = args.value_of("match") {
        let is_sensitive = !args.is_present("i");
        println!("Scanning drive {} with matcher `{}` ({})",
                 drive,
                 pattern,
                 if is_sensitive { "case-sensitive" } else { "case-insensitive" }
        );

        let options = MatchOptions {
            case_sensitive: is_sensitive,
            require_literal_leading_dot: false,
            require_literal_separator: false,
        };

        algorithm::run(drive, |path: &&PathBuf|
            Pattern::new(pattern)
                .expect("Illegal matcher syntax")
                .matches_path_with(&path.as_path(), options),
        );
    } else {
        println!("Scanning drive {}", drive);
        algorithm::run(drive, |_| true);
    }

    println!("Finished in {} seconds", instant.elapsed().as_secs_f32());
}