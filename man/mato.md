# matogro 1 29.11.2024 "MaTo Tools man-pages 0.1.1"

## NAME

matogro - transform markdown file to groff

## SYNOPSIS

`matogro` [-dhtVvX] [-l lang] [--lang lang] [-T device] [--device device] file

## DESCRIPTION

matogro parses markdown files and transoforms them
to groff format. Depending on the output device
selected - mom or man - macro specifc output is
generated.

## OPTIONS

- `-X, --dump-dot-file` Dumps the parsed syntax
  tree of the markdown input file to an outfile.
  The name of that file is the same as the input
  file, but the suffix is replaced with `dot`.
