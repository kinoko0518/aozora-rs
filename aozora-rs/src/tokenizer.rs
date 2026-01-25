mod command;
pub mod definition;
mod parser;

pub mod prelude {
    pub use super::command::{
        Note, SandwichedBegin, backref::BackRefKind, multiline::MultiLine, sandwiched::Sandwiched,
        single::Single, wholeline::WholeLine,
    };
    pub use super::definition::*;
    pub use super::parser::tokenize;
}
