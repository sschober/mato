# Architecture

```mermaid
classDiagram
    matogro --> Parser
    matogro --> Groff
    Parser --> Exp
    Renderer <|-- Groff
    Renderer <|-- Tex
    matote --> Parser
    matote --> Tex
```