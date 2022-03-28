use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::vec::IntoIter;
use crate::html::element::Element;
use crate::html::tag::Tag;

pub struct Document {
    root: Element,
}

impl Document {
    pub fn new() -> Self {
        let root = Element::Tag(
            Tag::new("html")
                .with_child(
                    Tag::new("head")
                )
                .with_child(
                    Tag::new("body")
                )
        );

        Self { root }
    }

    pub fn element_by_id<B: Borrow<str> + ?Sized>(&self, id: &B) -> Option<&Tag> {
        let found = find_element(&self.root, |e| match e {
            Element::Text(_) => false,
            Element::Tag(tag) => match tag.id() {
                None => false,
                Some(subject_id) => subject_id == id.borrow(),
            },
        });

        match found {
            None => None,
            Some(Element::Tag(tag)) => Some(tag),
            Some(Element::Text(_)) => unreachable!(),
        }
    }

    pub fn element_by_id_mut<B: Borrow<str> + ?Sized>(&mut self, id: &B) -> Option<&mut Tag> {
        let found = find_element_mut(&mut self.root, |e| match e {
            Element::Text(_) => false,
            Element::Tag(tag) => match tag.id() {
                None => false,
                Some(subject_id) => subject_id == id.borrow(),
            },
        });

        match found {
            None => None,
            Some(Element::Tag(tag)) => Some(tag),
            Some(Element::Text(_)) => unreachable!(),
        }
    }

    pub fn elements_by_class<B: Borrow<str> + ?Sized>(&self, class: &B) -> Vec<&Tag> {
        let found = find_all_elements(&self.root, |e| match e {
            Element::Text(_) => false,
            Element::Tag(tag) => tag.is_class(class.borrow()),
        });

        return found.into_iter().map(
            |x| match x {
                Element::Text(_) => unreachable!(),
                Element::Tag(t) => t,
            }
        ).collect();
    }
}

fn find_element(element: &Element, p: impl Fn(&Element) -> bool + Copy) -> Option<&Element> {
    if p(element) { return Some(element); }

    if let Element::Tag(tag) = element {
        for e in tag.content() {
            if let Some(e) = find_element(e, p) {
                return Some(e);
            }
        }
    }

    return None;
}

fn find_element_mut(element: &mut Element, p: impl Fn(&Element) -> bool + Copy) -> Option<&mut Element> {
    if p(element) { return Some(element); }

    match element {
        Element::Text(_) => (),
        Element::Tag(tag) => {
            for e in tag.content_mut() {
                if let Some(e) = find_element_mut(e, &p) {
                    return Some(e);
                }
            }
        }
    }

    return None;
}

fn find_all_elements(element: &Element, p: impl Fn(&Element) -> bool) -> Vec<&Element> {
    let mut found = Vec::new();

    if p(element) {
        found.push(element);
    }

    if let Element::Tag(tag) = element {
        for child in tag.content() {
            found.append(&mut find_all_elements(child, &p));
        }
    }

    return found;
}

impl FromStr for Document {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(
            Document {
                root: s.parse()?,
            }
        )
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root)
    }
}

