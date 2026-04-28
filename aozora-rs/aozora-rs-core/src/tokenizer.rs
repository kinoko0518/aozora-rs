#![doc = include_str!("../docs/tokenize.md")]

mod annotation;
mod definition;
mod parser;
#[cfg(test)]
mod test;

pub use annotation::{
    Annotation, SandwichedBegin, backref::BackRefKind, multiline::MultiLine,
    sandwiched::Sandwiched, single::Single, wholeline::WholeLine,
};
pub use definition::{AozoraTokenKind, Tokenized};
pub use parser::tokenize;
