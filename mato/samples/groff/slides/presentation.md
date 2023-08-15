# Slides with MaTo
## How to create great presentations







//.QUAD RIGHT
_Sven Schober_
//.QUAD RIGHT
_15.08.2023_

//.NEWSLIDE

# Preamble

You have to provide a `preamble.mom`
```
.TITLE "Slides with MaTo"
\&.PDF_TITLE "\*[$TITLE]"
\&.DOCTYPE SLIDES \\ 
ASPECT 16:9 \ 
HEADER "\*[$TITLE]" "" "" \\ 
FOOTER "Sven Schober" "" "\*S[+2]\*[SLIDE#]\*S[-2]" \\ 
TRANSITION "Box 1 . 0" \\ 
PAUSE "Wipe 1"
```

Therein define title, header and footer.
//.NEWSLIDE
# Preamble (2)

Define font and heading sizes:
```
.FAMILY Minion
\&.PT_SIZE 18
\&.HEADING_STYLE 1 CAPS FONT R QUAD LEFT
\&.HEADING_STYLE 2 CAPS FONT R SIZE -4  QUAD LEFT
\&.HEADING_STYLE 3 FONT I SIZE -1 QUAD LEFT

```

This sets the font to Minion and makes headers level 1 and 2 all caps.

Also left aligns them (central alignment is default in mom slides).

//.NEWSLIDE
# Preamble (3)

Define code style 
```
.QUOTE_STYLE QUAD LEFT
\&.CODE_SIZE 80
\&.CODE_FAMILY IosevkaTerm

```

Quad left aligns the listings on the left.

//.NEWSLIDE
# Title Page

Use level 1 header as title

//.NEWSLIDE

# Code Listings

Here we can see a simple script:

```
$ echo "hello world"
hello wordl
```

