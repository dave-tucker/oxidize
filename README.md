oxidize
=======

## What is oxidize

`oxidize` is designed to be a fast, flexible, drop-in replacement for GNU Make.
As the Makefile syntax isn't exactly well-defined (there is no grammar for it and one will never be developed) it may not be possible to support 100% of all the Makefiles out there.
I hope that it should be able to cover 80% of the use cases.

## Current Status

This project is pre-alpha and under active development.

`oxidize` can:

- Read the two example Makefiles in the `assets` directory. These were taken from the GNU Make Manual.
- The output of the `oxidize` can be piped to `dot` to display the DAG that was assembled from the Makefile

`oxidize` cannot:
- Read all Makefiles
- Actually build anything!

## Why replace make?

GNU Make is a tool which controls the generation of executables and other non-source files of a program from the program's source files.

While Make works well, it's not a "modern" developer tool - Make 4.0 was released in 2013 (6 years ago at the time of writing).

The areas where I feel that it could use improvement are:

- Windows support
- User configuration (e.g colored output, default shells etc...)
- Ease of extension and trying new features (i.e support for Docker containers)
- Easy to debug/trace/graph
- Nicer documentation (searchable)

Adding all this to GNU Make would be hard as it's a large legacy C code base (42K lines)