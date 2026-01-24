mod deco;
mod nihongo;
pub mod scopenizer;
pub mod tokenizer;

pub mod prelude {
    use winnow::LocatingSlice;

    pub type Input<'s> = LocatingSlice<&'s str>;
    pub type Span = std::ops::Range<usize>;

    pub use crate::deco::*;

    pub use crate::scopenizer::definition::{FlatToken, Scope};
    pub use crate::scopenizer::prelude::scopenize;

    pub use crate::tokenizer::definition::{AozoraToken, AozoraTokenKind};
    pub use crate::tokenizer::prelude::tokenize;
}
