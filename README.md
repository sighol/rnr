# RnR
[![Build Status](https://travis-ci.org/ChuckDaniels87/rnr.svg?branch=master)](https://travis-ci.org/ChuckDaniels87/rnr)
[![Build status](https://ci.appveyor.com/api/projects/status/97e28mxlakxbeqex/branch/master?svg=true)](https://ci.appveyor.com/project/ChuckDaniels87/rnr/branch/master)
[![Crates.io](https://img.shields.io/crates/v/rnr.svg)](https://crates.io/crates/rnr)
[![License](https://img.shields.io/crates/l/rnr.svg)](https://github.com/ChuckDaniels87/rnr/blob/master/LICENSE)

*RnR* is a command-line tool to **securely rename** multiple files and directories that
supports regular expressions.

## Features
* Batch rename files and directories.
* Automated checks to avoid unwanted file collisions, removals or overwrites.
* Use regexp, including capture groups.
* Include directories recursively.
* Create backup files.
* Create and read operations from dump file.
* Undo operations from dump file.
* Exclude/include hidden files.
* Linux, Mac and Windows support, including terminal coloring.
* Extensive unit testing.

# Install

## Binaries

### GitHub Releases
You can download binaries from [latest release
page](https://github.com/ChuckDaniels87/rnr/releases), choose the compressed
file corresponding to your platform. These compressed files contain the
executable and other additional content such as completion files (*Bash*, *Zsh*,
*fish* and *PowerShell*).

### Arch Linux
A package is available in the AUR
([`rnr`](https://aur.archlinux.org/packages/rnr/)) to install latest version of
*RnR* on Arch Linux.

## From Source
*RnR* is written in Rust. You can build it from source using Cargo.

### From git repository
```sh
git clone https://github.com/ChuckDaniels87/rnr .
cargo install
```
### From Crates.io
```sh
cargo install rnr
```
# Usage
## Options
```
USAGE:
    rnr [FLAGS] [OPTIONS] <EXPRESSION> <REPLACEMENT> <PATH(S)>...
    rnr [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -b, --backup          Generate file backups before renaming
    -n, --dry-run         Only show what would be done (default mode)
        --dump            Force dumping operations into a file even in dry-run mode
    -f, --force           Make actual changes to files
    -h, --help            Prints help information
    -x, --hidden          Include hidden files and directories
    -D, --include-dirs    Rename matching directories
        --no-dump         Do not dump operations into a file
    -r, --recursive       Recursive mode
    -s, --silent          Do not print any information
    -V, --version         Prints version information

OPTIONS:
        --color <color>        Set color output mode [default: auto]  [possible values: always, auto, never]
    -d, --max-depth <LEVEL>    Set max depth in recursive mode

ARGS:
    <EXPRESSION>     Expression to match (can be a regex)
    <REPLACEMENT>    Expression replacement
    <PATH(S)>...     Target paths

SUBCOMMANDS:
    from-file    Read operations from a dump file
    help         Prints this message or the help of the given subcommand(s)
```

## Default behavior
* Checks all operations to avoid overwriting existing files.
* *Dry-run* by default.
* Only **UTF-8 valid** input arguments and filenames.
* Works on files and symlinks (ignores directories).
* Accepts multiple files as arguments.
* Accepts a **regex** to generate matches. These expressions have same
  limitations of `regex` crate. You can check regex syntax
  [here](https://docs.rs/regex/#syntax). It supports numbered and named *capture
  groups*.
* If max depth is not provided to recursive mode, it is assumed *infinite*.
* Does not generate backups.
* Output is *colored* (only ANSI terminals).
* Ignore hidden files and directories.
* Dump all operations into a file in force mode. This dump file can be used to
  undo these operations from `from-file` subcommand.

## Demo
[![Demo](https://cdn.rawgit.com/ChuckDaniels87/b0607fdaa44c6201cde398b6a9e23e4e/raw/59d43365d15c55d9c259edd29292609c06de21f7/rnr-demo.svg)](https://cdn.rawgit.com/ChuckDaniels87/b0607fdaa44c6201cde398b6a9e23e4e/raw/f29d84760f4225dce74bf81052180e12a287b892/rnr-demo.svg)

## Examples
* [Rename a list of files](#rename-a-list-of-files)
    * [Include directories](#include-directories)
    * [Combination with other UNIX tools](#combination-with-other-unix-tools)
* [Recursive rename](#recursive-rename)
    * [Recursive rename with max directory depth](#recursive-rename-with-max-directory-depth)
    * [Recursive rename including directories and hidden files](#recursive-rename-including-directories-and-hidden-files)
* [Undo/redo operations using dump file](#undoredo-operations-using-dump-file)
* [Create backup files before renaming](#create-backup-files-before-renaming)
* [Advanced regex examples](#advanced-regex-examples)
    * [Replace extensions](#replace-extensions)
    * [Replace numbers](#replace-numbers)
    * [Capture groups](#capture-groups)
    * [Capture several named groups and swap them](#capture-several-named-groups-and-swap-them)

### Rename a list of files
You can pass a list of files to be renamed as arguments:
```sh
rnr -f file renamed ./file-01.txt ./one/file-02.txt ./one/file-03.txt
```
*Original tree*
```
.
├── file-01.txt
├── file-02.txt
├── file-03.txt
└── one
    ├── file-01.txt
    ├── file-02.txt
    └── file-03.txt
```
*Renamed tree*
```
.
├── renamed-01.txt
├── file-02.txt
├── file-03.txt
└── one
    ├── file-01.txt
    ├── renamed-02.txt
    └── renamed-03.txt
```

#### Include directories
Directories are ignored by default but you can also include them to be renamed using the option `-D`.
```
rnr -f -D foo bar ./*
```
*Original tree*
```
.
├── foo
│   └── foo.txt
└── foo.txt
```
*Renamed tree*
```
.
├── bar
│   └── foo.txt
└── bar.txt
```

#### Combination with other UNIX tools
You can combine `rnr` with other UNIX tools using pipes to pass arguments.

##### Find files older than 1 day and rename them
```sh
find . -type f +mtime 1 | xargs rnr -f file renamed
```

##### Read list of files from a file
```sh
cat file_list.txt | xargs rnr -f file rename
```

`file_list.txt` content:
```
file-01.txt
one/file-02.txt
one/file-03.txt
```

### Recursive rename
If recursive (`-r`) option is passed, `rnr` will look for al files in the path recursively without depth limit.
```sh
rnr -f -r file renamed ./
```
*Original tree*
```
.
├── file-01.txt
├── file-02.txt
├── file-03.txt
└── one
    ├── file-01.txt
    ├── file-02.txt
    ├── file-03.txt
    └── two
        ├── file-01.txt
        ├── file-02.txt
        ├── file-03.txt
        └── three
            ├── file-01.txt
            ├── file-02.txt
            └── file-03.txt
```
*Renamed tree*
```
.
├── renamed-01.txt
├── renamed-02.txt
├── renamed-03.txt
└── one
    ├── renamed-01.txt
    ├── renamed-02.txt
    ├── renamed-03.txt
    └── two
        ├── renamed-01.txt
        ├── renamed-02.txt
        ├── renamed-03.txt
        └── three
            ├── renamed-01.txt
            ├── renamed-02.txt
            └── renamed-03.txt
```
#### Recursive rename with max directory depth
Similarly, you can set a maximum directory depth in combination with recursive operations.
```sh
rnr -f -r -d 2 file renamed ./
```
*Original tree*
```
.
├── file-01.txt
├── file-02.txt
├── file-03.txt
└── one
    ├── file-01.txt
    ├── file-02.txt
    ├── file-03.txt
    └── two
        ├── file-01.txt
        ├── file-02.txt
        ├── file-03.txt
        └── three
            ├── file-01.txt
            ├── file-02.txt
            └── file-03.txt
```
*Renamed tree*
```
.
├── renamed-01.txt
├── renamed-02.txt
├── renamed-03.txt
└── one
    ├── renamed-01.txt
    ├── renamed-02.txt
    ├── renamed-03.txt
    └── two
        ├── file-01.txt
        ├── file-02.txt
        ├── file-03.txt
        └── three
            ├── file-01.txt
            ├── file-02.txt
            └── file-03.txt
```

#### Recursive rename including directories and hidden files
`rnr` ignore hidden files by default to speed up the operations and avoid problems with some particular directories like `.git/` or `.local/`. You can include hidden files passing `-x` option. Also, you can use include directories `-D` option with `-r` too.
```sh
rnr -f -r
```
*Original tree*
```
.
├── .foo_hidden_file.txt
├── foo.txt
├── foo
│   ├── foo.txt
│   └── foo
│       └── foo.txt
└── .foo_hidden_dir
    └── foo.txt
```
*Renamed tree*
```
.
├── .bar_hidden_file.txt
├── bar.txt
├── bar
│   ├── bar.txt
│   └── bar
│       └── bar.txt
└── .bar_hidden_dir
    └── bar.txt
```

### Undo/redo operations using dump file
When you perform a renaming operation, `rnr` will create by default a dump file in the current directory you executed the command. This file can be used to easily revert the operations using `from-file` and `-u` option.

*Rename operation*
```sh
rnr -f foo bar ./*
```
*Undo previous operation*
```sh
rnr from-file -f -u rnr-[timestamp].json
```

If you want to redo the operation just pass the dump file without any additional argument:
```sh
rnr from-file -f rnr-[timestamp].json

```

### Create backup files before renaming
`rnr` can create backup files before renaming for any operation passing `-b` option. The backup files names are ensured to be unique and won't be overwritten if another backup is created. If you are working with many large files, take into account that files will be duplicated.

```sh
rnr -f -b file renamed ./*
```

*Original tree*
```
.
├── file-01.txt
├── file-02.txt
└── file-03.txt
```
*Renamed tree*
```
.
├── file-01.txt.bk
├── file-02.txt.bk
├── file-03.txt.bk
├── renamed-01.txt
├── renamed-02.txt
└── renamed-03.txt
```

### Advanced regex examples
More info about regex used [in the `regex` package](https://docs.rs/regex).
#### Replace extensions
```
rnr -f '\..*$' '.txt' ./*
```
*Original tree*
```
.
├── file-01.ext1
├── file-02.ext2
└── file-03.ext3
```
*Renamed tree*
```
.
├── file-01.txt
├── file-02.txt
└── file-03.txt
```

#### Replace numbers
```
rnr -f '\d' '1' ./*
```
*Original tree*
```
.
├── file-01.txt
├── file-02.txt
└── file-03.txt
```
*Renamed tree*
```
.
├── file-11.txt
├── file-12.txt
└── file-13.txt
```
#### Capture groups
1. Capture three unnamed groups [`name(1)-number(2).extension(3)`].
2. Swap group 1 (name) and group 2 (number).
```sh
rnr -f '(\w+)-(\d+).(\w+)' '${2}-${1}.${3}' ./*
```
*Original tree*
```
.
├── file-01.txt
├── file-02.txt
└── file-03.txt
```
*Renamed tree*
```
.
├── 01-file.txt
├── 02-file.txt
└── 03-file.txt
```
#### Capture several named groups and swap them
1. Capture two digits as `number`.
2. Capture extension as `ext`.
3. Swap groups.
```sh
rnr -f '(?P<number>\d{2})\.(?P<ext>\w{3})' '${ext}.${number}' ./*
```
*Original tree*
```
.
├── file-01.txt
├── file-02.txt
└── file-03.txt
```
*Renamed tree*
```
.
├── file-txt.01
├── file-txt.02
└── file-txt.03
```

#### Auto-generated running number
The special string `$i` in the replacement will be replaced by a running number
that starts with 1.
```sh
rnr -f '.*' 'my file $i.txt' ./*
```
*Original tree*
```
.
├── file-01.txt
├── file-02.txt
└── file-03.txt
```
*Renamed tree*
```
.
├── my file 01.txt
├── my file 02.txt
└── my file 03.txt
```