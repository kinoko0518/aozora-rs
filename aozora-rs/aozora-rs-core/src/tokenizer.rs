pub mod definition;
mod note;
mod parser;

pub use definition::{AozoraTokenKind, Tokenized};
pub use note::{
    Note, SandwichedBegin, backref::BackRefKind, multiline::MultiLine, sandwiched::Sandwiched,
    single::Single, wholeline::WholeLine,
};
pub use parser::tokenize;
