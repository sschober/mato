//! capture the essence of a markdown abstract syntax tree
//! 
/// Expressions are the building blocks of an abstract syntax tree
#[derive(Debug)]
pub enum Exp {
    /// A literal is a string rendered as is
    Literal(String),
    /// An escaped literal probabely has to be treated in a special way, depending on the rendering backend
    EscapeLit(String),
    /// Section headers with a separate parameter specifying the level
    Heading(Box<Exp>, u8),
    /// Encapsulates boldness; can contain various other formattings 
    Bold(Box<Exp>),
    /// Encapsulates cursiveness; can contain varios other formattings
    Italic(Box<Exp>),
    /// Encapsulates text rendered in non-proportional font, usually used for computer code
    Teletype(Box<Exp>),
    Quote(Box<Exp>),
    Footnote(Box<Exp>),
    HyperRef(Box<Exp>, Box<Exp>),
    // this enables composition, forming the tree
    Cat(Box<Exp>, Box<Exp>),
    // this is a neutral element, yielding no ouput
    Empty(),
}


impl Exp {
    /// constructs new Exp of self and expr
    pub fn cat(self, expr: Exp) -> Exp {
        Exp::Cat(Box::new(self), Box::new(expr))
    }
}

// TODO all these to_string invocation incur a copy!
pub fn lit(s: &str) -> Exp {
    Exp::Literal(s.to_string())
}
pub fn escape_lit(s: &str) -> Exp {
    Exp::EscapeLit(s.to_string())
}
pub fn heading(exp: Exp, lvl: u8) -> Exp {
    Exp::Heading(Box::new(exp), lvl)
}
pub fn footnote(exp: Exp) -> Exp {
    Exp::Footnote(Box::new(exp))
}
pub fn hyperref(exp1: Exp, exp2: Exp) -> Exp {
    Exp::HyperRef(Box::new(exp1), Box::new(exp2))
}
