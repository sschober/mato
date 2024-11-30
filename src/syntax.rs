//! capture the essence of a markdown abstract syntax tree
//!

use std::fmt;

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
    /// Separate consequential paragraphs
    Paragraph(),
    /// code and stuff
    PreformattedLiteral(String),
    /// A literal is a string rendered as is
    Literal(String),
    /// An escaped literal probably has to be treated in a special
    /// way, depending on the rendering backend
    EscapeLit(String),
    /// A dropping capital, usually found at the beginning of chapters
    /// lowering down given amount of lines
    DropCap(u8, u8),
    // A color specification
    Color(Box<Tree>),
    /// Most often a single digit signifying the chapter number, and a color
    ChapterMark(Box<Tree>),
    /// Section headers with a separate parameter specifying the level
    Heading(Box<Tree>, u8, String),
    /// Encapsulates boldness; can contain various other formattings
    Bold(Box<Tree>),
    /// Encapsulates cursiveness; can contain varios other formattings
    Italic(Box<Tree>),
    /// Bold and Italic at the same time - nesting does not work here
    /// as we ne a special escape sequence to activate this: \*[BDI]
    BoldItalic(Box<Tree>),
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
    // web-link
    HyperRef(Box<Tree>, Box<Tree>),
    // document internal link
    DocRef(String, Box<Tree>),
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
    /// groff knows .SP instructions, which are important to insert
    /// after headings to introduce some vertical space after the heading
    VSpace(),
    // this is a neutral element, yielding no ouput
    Empty(),
}

impl Tree {
    /// constructs new Exp of self and expr
    #[must_use]
    pub fn cat(self, expr: Self) -> Self {
        Self::Cat(Box::new(self), Box::new(expr))
    }
    pub fn cat_box(self, expr: Box<Self>) -> Box<Self> {
        Box::new(Self::Cat(Box::new(self), expr))
    }
}

fn address_of(t: &Tree) -> String {
    format!("{:p}", t).strip_prefix("0").unwrap().to_owned()
}

/// incomplete display impl that generates graphviz dot
/// notation tree
impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tree::Cat(t1, t2) => {
                write!(
                    f,
                    "{}[label=\"||\"];\n{} -> {{{},{}}};\n{}\n{}",
                    address_of(self),
                    address_of(self),
                    address_of(t1),
                    address_of(t2),
                    *t1,
                    *t2
                )
            }
            Tree::Literal(s) => write!(
                f,
                "{} [label=\"l('{}')\"];",
                address_of(self),
                s.replace("\"", "")
            ),
            Tree::Document(_, t) => write!(
                f,
                "digraph graphname {{\n{}[label=\"Root\"];\n{} ->{};\n{}\n}}",
                address_of(self),
                address_of(self),
                address_of(t),
                t
            ),
            Tree::Paragraph() => write!(f, "{} [label=\"P\"];", address_of(self)),
            Tree::PreformattedLiteral(_) => todo!(),
            Tree::EscapeLit(_) => todo!(),
            Tree::DropCap(_, _) => todo!(),
            Tree::Color(_) => todo!(),
            Tree::ChapterMark(_) => todo!(),
            Tree::Heading(t, lvl, _) => {
                write!(
                    f,
                    "{} [label=\"H {}\"];\n{} -> {};\n{}",
                    address_of(self),
                    lvl,
                    address_of(self),
                    address_of(t),
                    *t
                )
            }
            Tree::Bold(t) => write!(
                f,
                "{} [label=\"B\"];\n{} -> {};\n{}",
                address_of(self),
                address_of(self),
                address_of(t),
                *t
            ),
            Tree::Italic(t) => write!(
                f,
                "{} [label=\"I\"];\n{} -> {};\n{}",
                address_of(self),
                address_of(self),
                address_of(t),
                *t
            ),
            Tree::BoldItalic(_) => todo!(),
            Tree::SmallCaps(_) => todo!(),
            Tree::CodeBlock(_, _) => todo!(),
            Tree::InlineCode(t) => write!(
                f,
                "{} [label=\"C\"]; {} -> {};\n{}",
                address_of(self),
                address_of(self),
                address_of(t),
                *t
            ),
            Tree::Quote(_) => todo!(),
            Tree::Footnote(_) => todo!(),
            Tree::RightSidenote(_) => todo!(),
            Tree::HyperRef(_, _) => todo!(),
            Tree::DocRef(_, _) => todo!(),
            Tree::List(_, _) => todo!(),
            Tree::ListItem(_, _) => todo!(),
            Tree::MetaDataBlock(_) => todo!(),
            Tree::MetaDataItem(_, _) => todo!(),
            Tree::ImageSizeSpec(_, _) => todo!(),
            Tree::Image(_, _, _) => todo!(),
            Tree::LineBreak() => write!(f, "{} [label=\"\\\\n\"]", address_of(self)),
            Tree::VSpace() => write!(f, "{} [label=\"V\"]", address_of(self)),
            Tree::Empty() => write!(f, "{} [label=\"\"]", address_of(self)),
        }
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
pub fn heading(exp: Tree, lvl: u8, name: &str) -> Tree {
    Tree::Heading(Box::new(exp), lvl, name.to_string())
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
