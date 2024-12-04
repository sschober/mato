# matogro 1 29.11.2024 "MaTo Tools man-pages 0.1.1"

## NAME

matogro - transform markdown file to groff

## SYNOPSIS

`matogro` [*-dhtVvX*] [*-l* _lang_] [*--lang* _lang_] [*-T* _device_] [*--device* _device_] _file_

`matogro` **-h|--help**

`matogro` **-v|--version**

## DESCRIPTION

**matogro** parses markdown files and transforms them
to groff format. Depending on the output device
selected with the **-T** option - mom or man -
macro specifc output is generated.

Specific markdown conventions have to be followed
in order to get adequate results, when rendering
to groff man.

A first level heading line, introduced by a single
pound character at the beginning of a line, will
create the man page header line.

## OPTIONS

- `-d, --debug`
  Enable debug log level, which is one level above
  _verbose_, producing more fine grained output.

- `-h, --help`
  Output help message and exit.

- `-X, --dump-dot-file`
  Dumps the parsed syntax
  tree of the markdown input file to an outfile.
  The name of that file is the same as the input
  file, but the suffix is replaced with `dot`.
