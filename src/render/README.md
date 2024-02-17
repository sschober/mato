# Mato - Renderer

In this directory, you can find the rendering backend of mato.

Currently, there is a [`markdown`](markdown.rs) renderer, which is used in 
`matofmt` to spill out `markdown` again, producing a formatting 
effect.

The second renderer is the [`mom`](groff/mom.rs) renderer,
which produces groff source, that uses the `mom` macro
package to produce nicely formatted PDFs.