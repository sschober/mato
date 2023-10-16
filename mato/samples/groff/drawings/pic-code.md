# PIC images in markdown
>>(p)
The following `pic` code block is being passed to the `pic` binary and its output put in place at the exact location of the code:

``` pic
box "doc" "<md>";
arrow;
ellipse "parser";
arrow;
box "AST";
```

The rendered output looks like this:

```pic
box "doc" "<md>";
arrow;
ellipse "parser";
arrow;
box "AST";
```

You can see two boxes and one ellipse, connected by arrows.