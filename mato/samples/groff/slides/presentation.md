---
doctype: SLIDES
title: "Slides with MaTo"
subtitle: "How to create presentations"
author: "Sven Schober"
---
# Preamble

You have to provide a `preamble.mom`
```
\&.DOCTYPE SLIDES \\ 
ASPECT 16:9 \ 
HEADER "\*[$TITLE]" "" "" \\ 
FOOTER "Sven Schober" "" "\*S[+2]\*[SLIDE#]\*S[-2]" 
```

Therein define title, header and footer.

# Preamble (2)

Define font and heading sizes:
```
.FAMILY Minion
\&.PT_SIZE 18
\&.HEADING_STYLE 1 CAPS FONT R QUAD LEFT
\&.HEADING_STYLE 2 CAPS FONT R SIZE -4  QUAD LEFT
\&.HEADING_STYLE 3 FONT I SIZE -1 QUAD LEFT
\&.QUAD LEFT
```

This sets the font to Minion and makes headers level 1 and 2 all caps.

Also left aligns them (central alignment is default in mom slides).

Sets general alignment to left.

# Preamble (3)

Define code style 
```
.QUOTE_STYLE QUAD LEFT
\&.CODE_SIZE 80
\&.CODE_FAMILY IosevkaTerm
\&.START

```

Quad left aligns the listings on the left.

Finally, starts the document.

# Code Listings

Here we can see a simple script:

```
$ echo "hello world"
hello world
```

# Multicolumn

It is also possible to fill multiple columns.

Unfortunately, there currently is no syntax for this.

//.TAB_SET 1 1p 25P
//.TAB_SET 2 32P 25P
//.MCO
//.TAB 1

Here is stuff on the left

Even more stuff

A lot of stuff
//.MCR
//.TAB 2

And other stuff on the right

More other stuff

Last of other stuff
//.MCX

# Lists

You can also make bullet lists:

* A first item
* A second item
* ...and so on

And use formatting in the items:

* A *bold* item
  * An _italic_ item in a sublist
  * Another item of the sublist
* A `code` item back in the top level list

//.TOC
