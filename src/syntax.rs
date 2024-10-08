//! capture the essence of a markdown abstract syntax tree
//!

#[derive(Debug, Clone)]
pub enum DocType {
    DEFAULT,
    CHAPTER,
    SLIDES,
    LETTER,
}

/// Expressions are the building blocks of an abstract syntax tree
#[derive(Debug)]
pub enum Tree {
    Document(DocType, Box<Tree>),
    /// Separate consequential pargraps
    Paragraph(),
    /// code and stuff
    PreformattedLiteral(String),
    /// A literal is a string rendered as is
    Literal(String),
    /// An escaped literal probabely has to be treated in a special
    /// way, depending on the rendering backend
    EscapeLit(String),
    // A color specification
    Color(Box<Tree>),
    /// Most often a single digit signifying the chapter number, and a color
    ChapterMark(Box<Tree>),
    /// Section headers with a separate parameter specifying the level
    Heading(Box<Tree>, u8),
    /// Encapsulates boldness; can contain various other formattings
    Bold(Box<Tree>),
    /// Encapsulates cursiveness; can contain varios other formattings
    Italic(Box<Tree>),
    SmallCaps(Box<Tree>),
    /// Encapsulates code placed as a separate block, set apart from
    /// normal, flowing text
    CodeBlock(Box<Tree>, Box<Tree>),
    /// Encapsulates text rendered in non-proportional font, usually
    /// used for computer code, placed in line with normal text
    InlineCode(Box<Tree>),
    Quote(Box<Tree>),
    Footnote(Box<Tree>),
    RightSidenote(Box<Tree>),
    HyperRef(Box<Tree>, Box<Tree>),
    // this enables composition, forming the tree
    Cat(Box<Tree>, Box<Tree>),
    // Lists, should contain ListItems
    List(Box<Tree>, u8),
    // singular items of lists
    ListItem(Box<Tree>, u8),
    // captures a meta data block, basically a list of key values
    // like title, author etc.
    MetaDataBlock(Box<Tree>),
    // a singular meta data item
    MetaDataItem(String, String),
    // captures size specification in the of XxY
    ImageSizeSpec(Box<Tree>, Box<Tree>),
    /// image with caption, path, and image size spec
    Image(Box<Tree>, Box<Tree>, Box<Tree>),
    /// new line
    LineBreak(),
    // this is a neutral element, yielding no ouput
    Empty(),
}

impl Tree {
    /// constructs new Exp of self and expr
    #[must_use]
    pub fn cat(self, expr: Self) -> Self {
        Self::Cat(Box::new(self), Box::new(expr))
    }
}

// TODO all these to_string invocation incur a copy!
#[must_use]
pub fn lit(s: &str) -> Tree {
    Tree::Literal(s.to_string())
}
#[must_use]
pub fn prelit(s: &str) -> Tree {
    Tree::PreformattedLiteral(s.to_string())
}
#[must_use]
pub fn escape_lit(s: &str) -> Tree {
    Tree::EscapeLit(s.to_string())
}
#[must_use]
pub fn heading(exp: Tree, lvl: u8) -> Tree {
    Tree::Heading(Box::new(exp), lvl)
}
#[must_use]
pub fn color(exp: Tree) -> Tree {
    Tree::Color(Box::new(exp))
}
#[must_use]
pub fn footnote(exp: Tree) -> Tree {
    Tree::Footnote(Box::new(exp))
}
#[must_use]
pub fn hyperref(exp1: Tree, exp2: Tree) -> Tree {
    Tree::HyperRef(Box::new(exp1), Box::new(exp2))
}
#[must_use]
pub fn bold(exp: Tree) -> Tree {
    Tree::Bold(Box::new(exp))
}
#[must_use]
pub fn list(exp: Tree, level: u8) -> Tree {
    Tree::List(Box::new(exp), level)
}
#[must_use]
pub fn list_item(exp: Tree, level: u8) -> Tree {
    Tree::ListItem(Box::new(exp), level)
}
#[must_use]
pub fn meta_data_item(key: String, value: String) -> Tree {
    Tree::MetaDataItem(key, value)
}
#[must_use]
pub fn meta_data_block(exp: Tree) -> Tree {
    Tree::MetaDataBlock(Box::new(exp))
}
#[must_use]
pub fn image_size(x: Tree, y: Tree) -> Tree {
    Tree::ImageSizeSpec(Box::new(x), Box::new(y))
}
#[must_use]
pub fn image(caption: Tree, path: Tree, size_spec: Tree) -> Tree {
    Tree::Image(Box::new(caption), Box::new(path), Box::new(size_spec))
}
#[must_use]
pub fn empty() -> Tree {
    Tree::Empty()
}
pub fn cat(t1: Tree, t2: Tree) -> Tree {
    Tree::Cat(Box::new(t1), Box::new(t2))
}
