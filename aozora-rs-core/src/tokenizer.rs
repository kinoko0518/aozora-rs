pub mod definition;
mod note;
mod parser;

pub mod prelude {
    pub use super::definition::*;
    pub use super::note::{
        Note, SandwichedBegin, backref::BackRefKind, multiline::MultiLine, sandwiched::Sandwiched,
        single::Single, wholeline::WholeLine,
    };
    pub use super::parser::tokenize_nometa;
}
