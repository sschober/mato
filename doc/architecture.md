# Architecture
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