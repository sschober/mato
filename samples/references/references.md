# Mato

>>(\{mato_dark_blue}m)

## Introduction

%In the following we describe, the motivation,
architecture and implementation of `mato`.
`mato` is a set of tools to transform markdown
formatted text into pdf files. It uses `groff`,
GNU roff, for the production of pdfs.

### Motivation /moti/

%Here in this section, we present the motivation
for creating `mato`. Please see [later](#arch)

#### Rendering performance

Using `groff` was primarily motivated by its
very quick rendering times, as opposed to LaTeX
for example. A prior version of `mato` even used
LaTeX as a backend, or rendering engine. But upon
trying `groff` it quickly became apparent, that
it gave much better response times.

#### Installation size

But in addition to that, installation size of the
rendering backend dependencies drove the decision
to use `groff`. Modern LaTe distributions, like
TeX-Live can easily take up above 1GB of disk 
space and be very unwieldy to handle.

#### Community

Another point, why we chose `groff` is its really
active and live community. The mailing list has
constant activity and since quite some time the
code base is being developed rather constantly.

Bugs that were reported are being addressed 
thoroughly and quickly. And discussions are 
friendly and in a welcoming tone.

## Architecture /arch/

In the [previous section](#into), we presented the 
motivation, why we created `mato`.

## Implementation /impl/

## Conclusion
