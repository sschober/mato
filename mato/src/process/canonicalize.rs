use std::collections::HashMap;

use super::Process;

use crate::Exp;
use crate::syntax::meta_data_block;

pub struct Canonicalizer {}

/// descents the complete AST and erazes Empty() nodes
fn erase_empty(exp: Exp) -> Exp {
    match exp {
        Exp::Cat(b_exp1, b_exp2) => match *b_exp1 {
            Exp::Empty() => erase_empty(*b_exp2),
            _ => erase_empty(*b_exp1).cat(erase_empty(*b_exp2)),
        },
        Exp::MetaDataBlock(b_exp) => meta_data_block(erase_empty(*b_exp)),
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

    fn get_context(& mut self) -> std::collections::HashMap<String,String> {
        HashMap::new()
    }
}
