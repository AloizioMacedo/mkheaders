# Introduction

mkheaders is a CLI app for prepending headers to files.

It is an idempotent application, which is to say that if you run it two times in a row, it is the same as running just one time: it **won't** add two headers. This is an advantage relative to usual scripts that do prepends.

Moreover, it runs as a multi-threaded application with rayon for improved performance.

It is intended to be safe, and it creates a temporary file that only replaces the original one after it is completely filled up. So, theoretically, the worst case when using the app would be to have some dangling temporary files in the folder if you interrupt the execution. (In the future, we will add responses to some signals in order to do some cleanup when possible.)

Nevertheless, **please keep your files under version control** and/or have backups before using the script, particularly because it is still under development.

## Usage

To add headers to all files inside of a folder called "dir" recursively, run

```console
~$ mkheaders header_file.txt dir -r
```

To restrict the previous example to only all .py files, run

```console
~$ mkheaders header_file.txt dir -r -m ".*.py$"
```

For more information and options, run

```console
~$ mkheaders -h
Idempotent header prepender

Usage: mkheaders.exe [OPTIONS] <HEADER_FILE> <TARGET_FOLDER>

Arguments:
  <HEADER_FILE>    File containing the header
  <TARGET_FOLDER>  Target folder containing files to add header

Options:
  -m, --matching <MATCHING>  Regex to match file names that will be considered for the headers
  -r, --recursive            Recursively runs through the target directory, visiting inner directories
  -d, --delete               Flag for deleting the header if it exists rather than prepending
  -h, --help                 Print help
```
