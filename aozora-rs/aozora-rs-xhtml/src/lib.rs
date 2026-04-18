mod definitions;
mod xhtmlnize;

pub use crate::xhtmlnize::retokenized_to_xhtml;
pub use definitions::*;

pub struct XHTMLResult {
    pub xhtmls: Vec<String>,
    pub dependency: Vec<String>,
    pub chapters: Vec<Chapter>,
}
