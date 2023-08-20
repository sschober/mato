use super::Process;

use crate::Exp;

pub struct Canonicalizer {}

/// descents the complete AST and erazes Empty() nodes
fn erase_empty(exp: Exp) -> Exp {
    match exp {
        Exp::Cat(b_exp1, b_exp2) => match *b_exp1 {
            Exp::Empty() => *b_exp2,
            _ => erase_empty(*b_exp1).cat(erase_empty(*b_exp2)),
        },
        _ => exp,
    }
}

impl Process for Canonicalizer {
    fn process(&mut self, exp: Exp) -> Exp {
        eprintln!("{:?}", exp);
        let canon = erase_empty(exp);
        eprintln!("{:?}", canon);
        canon
    }
}
