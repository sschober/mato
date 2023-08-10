# MaTo - MArkdown TO transformer framework

To test `matogro` use:

```
cargo run --bin matogro sample/src/index.md
```

This will result in a file called `sample/src/index.pdf` with the
rendering, if all went well.

## Installation

Just use 

```
cargo install --path mato
```

to install `matote` and `matogro` to your rust binary directory.
 
## Watch mode

There is also a super-duper-watch-mode, which can activated via
`-w`. Then, the source file will be watched and if written to
will be reprocessed.