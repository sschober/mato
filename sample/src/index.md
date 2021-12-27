# MaToTe - a markdown to tex transformer

This little binary is a transformer for markdown formatted text to LaTeX. It
features only a subset of markdown that I commonly use. It tries hard to strike
a balance between conformity to the markdown "standard" (whatever that is) and
practicality.

## Features

It supports different heading levels (currently only three) and a quotations
"style"^(and even footnotes!). Ampersands are special characters in LaTeX, so if
you use them in markdown, they have to be escaped in the tex output. Let's try
that out: A&B. Let's see, if that also works in teletype expressions: `A&B`. And
fat ones: *A&B*. And italics: _A&B_. And fat italics: *_A&B_*. Is there
something like fat italic teletype? `_*A&B*_`. -- It seems there is.

## Technical Details

It is written in rust, primarily for me to learn the language, but also for me
to learn about writing parsers.^(To that end, I read the book "Crafting
Interpreters" by Robert Nystrom, which I can highly recommend! It is really good
and I learned a lot.) Markdown does not lend itself so well, to write a parser,
as it seems to me. Don't try to squeeze anything out of me in the vicinity of
the chomsky hiearchy or context freeness. 

To tranform a given text in markdown just invoke the `matote` command, like so:
