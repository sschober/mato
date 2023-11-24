# MaTo - MArkdown TransfOrmer framework

With `mato`, you can transform markdown formatted text into PDFs, 
and even more.

```mermaid
graph LR
md -- matopdf --> PDF
```

Mato currently comprises a markdown parsing frontend part, 
and two backend rederers: one using LaTeX and one using `groff`.

```mermaid
graph LR
md -- matote --> TeX -- latex --> PDF
md -- matogro --> groff -- pdfmom --> PDF
```

Both work, but the groff backend is more in use by me. 

And, more
importantly, the groff backend is much quicker. It usually takes
only around 1 second to process the input and produce the resulting
PDF. 

The LaTeX backend in comparison takes many times as much and
is heavily dependent on the ctan packages you include in your
preamble.

### Styling

The style (think of padding, fonts, etc.) is configurable, depending
on the backend chosen. 

The `groff` backend has a default preamble
(see [here](mato/src/bin/default-preamble.mom)), which defines 
standard styles. Settings therein can be overwritten by placing a
`preamble.mom` file next to your markdown file.


## Build and run

To test `matogro`, the groff based transformer, use:

```
cargo run --bin matogro mato/sample/src/index.md
```

This will result in a file called `mato/sample/src/index.pdf` with the
rendering, if all went well.

To test the LaTeX backend-based transformation, `matote`, use:

```
cargo run --bin matote mato/sample/src/index.md
```

## Installation

Just use 

```
cargo install --path mato
```

to install `matote` and `matogro` to your rust binary directory.
 
## Watch mode

There is also a super-duper-watch-mode, which can activated via
`-w`. Then, the source file will be watched and if written to
will be reprocessed. This feature can be used to create a kind
of WYSIWYG experierence, when writing.

![WYSIWYG editing](doc/WYSIWYG-editing.png)

The processing and update time of the PDF is usually around and below 1 second, so this is not instantaaneous, but good enough.

In the image above, I use kitty and `termpdf.py` to display the
PDF side by side with the markdown source file.

## Developing

The implementation is done in rust, primarily for me to learn 
the language. So, if you find any non-idiomatic stuff, feel
free to create a pull request.

To start reading the code, you might jump into one of the 
binary sources, I'd recommend [mato/src/bin/matopdf.rs](mato/src/bin/matopdf.rs).

There, mato is used to create groff source code from markdown
markup and then `pdfmom`, a groff-based tool, is used to
created the final PDF:

```mermaid
graph LR
md["markdown sources"] -- mato --> groff["groff sources"] -- pdfmom --> PDF
```

The parser is located in [mato/src/parser.rs](mato/src/parser.rs).

# Author

Sven Schober <sv3sch@gmail.com>
