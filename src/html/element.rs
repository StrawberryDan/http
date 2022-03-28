use std::fmt::{Display, Formatter};
use crate::html::tag::Tag;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Element {
    Text(String),
    Tag(Tag),
}

impl Display for Element {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Element::Text(s) => write!(f, "{}", s)?,
            Element::Tag(t) => write!(f, "{}", t)?,
        }

        return Ok(());
    }
}
