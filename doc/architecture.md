# Architecture

At first, mato started out as a pipeline oriented system.
The source code would be read, passed through some filters
and during that filtering, the code is modified and finally
written out again by a backend, that does the rendering
into the target language.

That is why there were a LaTeX and a groff backend.

This is also very analogous to standard compiler layout.

Mato even has a intermediate format, the `Exp` tree.

It still follows that idea, but the backend is very much
focused on groff and some aspects of the target language
might have leaked into processing and parsing.

## Dynamic

```mermaid
graph LR
m[md] --> p
p(parsing) --> a[AST] --> p1(processing) --> b[AST] --> r(rendering)
r --> PDF
````

## Static

```mermaid
classDiagram
    matogro --> Parser
    matogro --> Groff
    Parser --> Exp
    Processor <|-- Canonicalizer
    Processor <|-- MetaDataExtractor
    Porcessor <|-- Chain
    Renderer <|-- Groff
    Renderer <|-- Tex
    matote --> Parser
    matote --> Tex
```