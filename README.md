# legacy-rsfs

This library assists in reading data from RuneScape's legacy file system.
It aims to support the file system between the years 2005-2007. 
Writing data to the file system is not supported at this time but will be supported in the future.

This library is currently being tested against revision 317.

## Features

* Cache File System
    * Reading
        * Index file reading
        * Archive reading and decoding
        * File entry name hashing
* BZIP2 and GZIP compression and decompression

## Task List

* Reading
    * Archive
        * [ ] Versions
* Writing
    * CRUD operations
        * [ ] Index
        * [ ] Index file entries
        * [ ] Archives


## Usage


#### Loading from a file path
```rust
use legacy-rsfs::filesystem::FileSystem;


let fs = FileSystem::new(your_path)?;
```

#### Reading data from the cache

Let's try to read a MIDI file from the cache:

```rust
use legacy-rsfs::filesystem::FileSystem;
use legacy-rsfs::index::IndexType;


let fs = FileSystem::new(your_path)?;
let file_entry_id: u32 = 17;
let read_data: Vec<u8> = fs.read(IndexType::MIDI, file_entry_id)?;
```

Note: files in indexes other than the `IndexType::ARCHIVE` are compressed with
GZIP. So in this case, the data that we just read (`read_data`) is compressed with GZIP.
#### Decompressing data

legacy-rsfs supports BZIP2 and GZIP for compression and decompression.

Using the example from [Reading data from the cache](#reading-data-from-the-cache), let's decompress it with GZIP:
```rust
use legacy-rsfs::compression;
use legacy-rsfs::filesystem::FileSystem;
use legacy-rsfs::index::IndexType;


let fs = FileSystem::new(your_path)?;
let file_entry_id: u32 = 17;
let read_data: Vec<u8> = fs.read(IndexType::MIDI, file_entry_id)?;
let decompressed_data: Vec<u8> = compression::decompress_gzip(read_data)?;
```

Now we have the vector of bytes for the MIDI file (`decompressed_data`), let's write it to our computer so we can listen to some nice RuneScape music:

```rust
use std::fs::File;
use std::io::Write;
use legacy-rsfs::compression;
use legacy-rsfs::filesystem::FileSystem;
use legacy-rsfs::index::IndexType;


let fs = FileSystem::new(your_path)?;
let file_entry_id: u32 = 17;
let read_data: Vec<u8> = fs.read(IndexType::MIDI, file_entry_id)?;
let decompressed_data: Vec<u8> = compression::decompress_gzip(read_data)?;
// now we write the bytes to a new file on our computer
let mut midi = File::create("17.mid")?;
midi.write_all(&decompressed_data)?;
```
#### Accessing archive data

Files in an `Archive` are compressed with BZIP2. 
You do not need to manually decompress files inside an `Archive` because
the library will automatically decompress them after accessing an `Archive`.

Let's try to get the data for the RuneScape logo:

```rust
let fs = FileSystem::new(your_path)?;
let title_archive_id = 1;
// let's open the Title archive
let archive: Archive = fs.read_archive(title_archive_id)?;
// now find the logo that is inside the archive by using its name
let logo_entry: &ArchiveEntry = archive.entry_name("logo.dat")?;
```
After accessing the logo entry, you can get its uncompressed bytes data and do whatever you want with it:

```rust
let logo_entry: &ArchiveEntry = archive.entry_name("logo.dat")?;
let uncompressed_bytes: &[u8] = logo_entry.get_uncompressed_data();
```

More usage information will come as the library gets updated.

## Acknowledgements
The following resources below have helped solidify my understanding of the RuneScape cache file system:

* [Vicis](https://github.com/apollo-rsps/Vicis)
* [scape-editor](https://github.com/scape-tools/scape-editor)
* [Displee's cache library](https://github.com/Displee/rs-cache-library)
* [scapefs](https://github.com/Velocity-/scapefs)
* [Commie's RuneScape 317 Documentation](https://sites.google.com/site/commiesrunescapedocumentation/)

## License
[MIT](https://choosealicense.com/licenses/mit/)
