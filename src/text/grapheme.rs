use std::rc::Rc;

use crate::style::Style;

/// A grapheme associated to a style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledGrapheme {
    pub symbol: Rc<String>,
    pub style: Style,
}
