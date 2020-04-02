# ddup

`ddup` (**D**etect **Dup**licates) is an extremely fast tool that identifies duplicated files in 
[Windows NTFS Volumes](https://en.wikipedia.org/wiki/NTFS).  
 
 ## Usage
 
 #### Scan C: recursively 
 ```shell script
ddup C:
```

#### Scan C: recursively, search for .dmp files (case-insensitive) 
```shell script
ddup C: -m **\*.dmp -i
```
Output:
```
Scanning drive C: with matcher `**\*.dmp` (case-insensitive)
Duplicated files [23134 bytes]
	1 C:\ProgramData\Microsoft\Windows\Containers\Dumps\g9292c13-143c-4070-98b5-7e12e2afddfc.dmp
	2 C:\Windows\LiveKernelReports\NDIS-30190504-0002.dmp
Finished in 7.4482803 seconds
```

## Installation

Install from crates.io *(Not yet available)*
```shell script
cargo install ddup
```

Install from repository:
```shell script
cargo install --path .
```

## Implementation

This tool is written in [Rust](https://www.rust-lang.org/) .

`ddup` obtains a recursive dirlist by leveraging the [NTFS USN Journal](https://en.wikipedia.org/wiki/USN_Journal) mechanism  
 in order to obtain USN records for [MFT](https://en.wikipedia.org/wiki/NTFS#Master_File_Table) (**M**aster **F**ile **T**able) entries.  

The Windows API is available via the following `IOCTL`s:
* [`FSCTL_ENUM_USN_DATA`](https://docs.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-fsctl_enum_usn_data)
* [`FSCTL_QUERY_USN_JOURNAL`](https://docs.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-fsctl_query_usn_journal)

The USN records represent either Files or Directories, linking one to another, so in order to resolve the full path  
of a file, we perform an SQL-equivalent "recursive join" operation on the records (implemented via a  `HashMap`).

After the full paths are resolved, we start comparing the files by using several iterations:
* Find groups of files that have the same size
* Compare files using fuzzy hashing

The results are most probably identical, although it is not strictly guaranteed.  
To guarantee total equivalence, use the `--strict` flag. *(Not Yet Available)*