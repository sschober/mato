//! capture the essence of a markdown abstract syntax tree
//!
/// Expressions are the building blocks of an abstract syntax tree
#[derive(Debug)]
pub enum Exp {
    Document(),
    /// Separate consequential pargraps
    Paragraph(),
    /// A literal is a string rendered as is
    Literal(String),
    /// An escaped literal probabely has to be treated in a special
    /// way, depending on the rendering backend
    EscapeLit(String),
    /// Most often a single digit signifying the chapter number
    ChapterMark(Box<Exp>),
    /// Section headers with a separate parameter specifying the level
    Heading(Box<Exp>, u8),
    /// Encapsulates boldness; can contain various other formattings
    Bold(Box<Exp>),
    /// Encapsulates cursiveness; can contain varios other formattings
    Italic(Box<Exp>),
    /// Encapsulates code placed as a separate block, set apart from
    /// normal, flowing text
    CodeBlock(Box<Exp>),
    /// Encapsulates text rendered in non-proportional font, usually
    /// used for computer code, placed in line with normal text
    InlineCode(Box<Exp>),
    Quote(Box<Exp>),
    Footnote(Box<Exp>),
    RightSidenote(Box<Exp>),
    HyperRef(Box<Exp>, Box<Exp>),
    // this enables composition, forming the tree
    Cat(Box<Exp>, Box<Exp>),
    // Lists, should contain ListItems
    List(Box<Exp>,u8),
    // singular items of lists
    ListItem(Box<Exp>,u8),
    // captures a meta data block, basically a list of key values
    // like title, author etc.
    MetaDataBlock(Box<Exp>),
    // a singular meta data item
    MetaDataItem(String, String),
    // this is a neutral element, yielding no ouput
    Empty(),
}

impl Exp {
    /// constructs new Exp of self and expr
    #[must_use]
    pub fn cat(self, expr: Self) -> Self {
        Self::Cat(Box::new(self), Box::new(expr))
    }
}

// TODO all these to_string invocation incur a copy!
#[must_use]
pub fn lit(s: &str) -> Exp {
    Exp::Literal(s.to_string())
}
#[must_use]
pub fn escape_lit(s: &str) -> Exp {
    Exp::EscapeLit(s.to_string())
}
#[must_use]
pub fn heading(exp: Exp, lvl: u8) -> Exp {
    Exp::Heading(Box::new(exp), lvl)
}
#[must_use]
pub fn footnote(exp: Exp) -> Exp {
    Exp::Footnote(Box::new(exp))
}
#[must_use]
pub fn hyperref(exp1: Exp, exp2: Exp) -> Exp {
    Exp::HyperRef(Box::new(exp1), Box::new(exp2))
}
#[must_use]
pub fn bold(exp: Exp) -> Exp {
    Exp::Bold(Box::new(exp))
}
#[must_use]
pub fn list(exp: Exp, level: u8) -> Exp {
    Exp::List(Box::new(exp), level)
}
#[must_use]
pub fn list_item(exp: Exp, level: u8) -> Exp {
    Exp::ListItem(Box::new(exp), level)
}
#[must_use]
pub fn meta_data_item(key: String, value: String) -> Exp {
    Exp::MetaDataItem(key, value)
}
#[must_use]
pub fn meta_data_block(exp: Exp) -> Exp {
    Exp::MetaDataBlock(Box::new(exp))
}
#[must_use]
pub fn empty() -> Exp {
    Exp::Empty()
}