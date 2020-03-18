# legacy-rsfs

This library assists in reading and modifying data from RuneScape's legacy file system.
It aims to support the file system between the years 2005-2007. 

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

```
TODO
```

## Acknowledgements
The following resources below have helped solidify my understanding of the RuneScape cache file system:

* [Vicis](https://github.com/apollo-rsps/Vicis)
* [scape-editor](https://github.com/scape-tools/scape-editor)
* [Displee's cache library](https://github.com/Displee/rs-cache-library)
* [scapefs](https://github.com/Velocity-/scapefs)
* [Commie's RuneScape 317 Documentation](https://sites.google.com/site/commiesrunescapedocumentation/)

## License
[MIT](https://choosealicense.com/licenses/mit/)