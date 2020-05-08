# ddup

![](https://github.com/netaneld122/ddup/workflows/Rust/badge.svg)

`ddup` (**D**etect **Dup**licates) is an extremely fast tool that identifies potentially duplicated files in 
[Windows NTFS Volumes](https://en.wikipedia.org/wiki/NTFS).

Note that since the NTFS Journal is limited in size, not **all** duplicated files will be found.

## Usage
 
#### Scan C: recursively and find duplicates
 ```
ddup C:
```

#### Scan C: recursively, search for duplicated .dmp files (case-insensitive) 
```
ddup C: -m **\*.dmp -i
```
Output:
```
Scanning drive C: with matcher `**\*.dmp` (case-sensitive) [Fuzzy comparison]
[1/3] Generating recursive dirlist
Finished in 7.798245 seconds
[2/3] Grouping by file size
Finished in 0.0028928 seconds
[3/3] Grouping by hash in thread pool
Potential duplicates [84654 bytes]
	C:\Windows\LiveKernelReports\NDIS-20190504-0002.dmp
	C:\ProgramData\Microsoft\Windows\Containers\Dumps\f9292c13-143c-4070-98b5-7e12e2afddfc.dmp
Finished in 0.001117 seconds
Overall finished in 7.857446 seconds
```

## Installation

Install from crates.io:
```shell script
cargo install ddup
```

Install from repository:
```shell script
cargo install --git https://github.com/netaneld122/ddup
```

## Implementation

This tool is written in [Rust](https://www.rust-lang.org/) .

`ddup` obtains a recursive dirlist by leveraging the [NTFS USN Journal](https://en.wikipedia.org/wiki/USN_Journal) mechanism 
in order to read USN records for [MFT](https://en.wikipedia.org/wiki/NTFS#Master_File_Table) (**M**aster **F**ile **T**able) entries.  

Windows USN Records can be fetched via the following `IOCTL`s:
* [`FSCTL_ENUM_USN_DATA`](https://docs.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-fsctl_enum_usn_data)
* [`FSCTL_QUERY_USN_JOURNAL`](https://docs.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-fsctl_query_usn_journal)

The USN records represent either Files or Directories, linking one to another, so in order to resolve the full path  
of a file, an SQL-equivalent "recursive join" has to be performed on the records (implemented via a `HashMap`).

After the full paths are resolved, we start comparing the files by using several iterations:
* Find groups of files that have the same size
* Compare files using fuzzy hashing on all cores simultaneously

The results are most probably identical, although it is not strictly guaranteed.  
To guarantee total equivalence, use the `--strict` flag (however this may impact performance greatly)

Note that due to the implementation's nature, `ddup` requires elevated Administrator privileges. 
