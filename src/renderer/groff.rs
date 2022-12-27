use crate::Exp;

use super::Renderer;

pub struct GroffRenderer;

impl Renderer for GroffRenderer {
    fn render(&self, exp: Exp) -> String {
        match exp {
            Exp::Literal(s) => format!("{}", s),
            Exp::Bold(b_exp) => format!("\\*[BOLDER]{}\\*[BOLDERX]", self.render(*b_exp)),
            Exp::Italic(b_exp) => format!("\\*[SLANT]{}\\*[SLANTX]", self.render(*b_exp)),
            Exp::Teletype(b_exp) => format!("\\*[CODE]{}\\*[CODE OFF]", self.render(*b_exp)),
            Exp::Heading(b_exp, level) => {
                format!(".HEADING {} \"{}\"", level + 1, self.render(*b_exp))
            }
            Exp::Quote(b_exp) => format!("\"{}\"", self.render(*b_exp)),
            Exp::Footnote(b_exp) => format!("\n.FOOTNOTE\n{}\n.FOOTNOTE END", self.render(*b_exp)),
            Exp::HyperRef(b_exp1, b_exp2) => format!(".PDF_WWW_LINK {} \"{}\"", self.render(*b_exp2), self.render(*b_exp1)),
            Exp::Cat(b_exp1, b_exp2) => format!("{}{}", self.render(*b_exp1), self.render(*b_exp2)),
            Exp::Empty() => format!(""),
        }
    }

}
