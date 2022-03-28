use std::collections::{HashMap, HashSet};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use crate::html::element::Element;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Tag {
    tag_name: String,
    attributes: HashMap<String, String>,
    content: Vec<Element>,
}

impl Tag {
    pub fn new<B: Borrow<str> + ?Sized>(tag_name: &B) -> Self {
        Self {
            tag_name: tag_name.borrow().to_string(),
            attributes: HashMap::new(),
            content: Vec::new(),
        }
    }

    pub fn attribute<B: Borrow<str> + ?Sized>(&self, key: &B) -> Option<&str> {
        self.attributes.get(key.borrow()).map(|s| s.as_str())
    }

    pub fn with_attribute<B: Borrow<str> + ?Sized>(mut self, key: &B, value: &B) -> Self {
        self.attributes.insert(key.borrow().to_string(), value.borrow().to_string());
        self
    }

    pub fn without_attribute<B: Borrow<str> + ?Sized>(mut self, key: &B) -> Self {
        self.attributes.remove(key.borrow());
        self
    }

    pub fn id(&self) -> Option<&str> {
        self.attribute("id")
    }

    pub fn with_id<B: Borrow<str> + ?Sized>(self, id: &B) -> Self {
        self.with_attribute("id", id.borrow())
    }

    pub fn classes(&self) -> HashSet<&str> {
        match self.attributes.get("class") {
            None => HashSet::new(),
            Some(classes) => classes.split_whitespace().collect(),
        }
    }

    pub fn is_class<B: Borrow<str> + ?Sized>(&self, class: &B) -> bool {
        return self.classes().contains(class.borrow())
    }

    pub fn with_class<B: Borrow<str> + ?Sized>(self, class: &B) -> Self {
        let current_class = self.attributes.get("class").map(|s| s.clone());

        match current_class {
            Some(c) => self.with_attribute("class", &format!("{} {}", c, class.borrow())),
            None => self.with_attribute("class", class.borrow()),
        }
    }

    pub fn content(&self) -> &Vec<Element> {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut Vec<Element> {
        &mut self.content
    }

    pub fn with_text<B: Borrow<str> + ?Sized>(mut self, text: &B) -> Self {
        self.content.push(Element::Text(text.borrow().to_string()));
        self
    }

    pub fn with_child(mut self, child: Tag) -> Self {
        self.content.push(Element::Tag(child));
        self
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}", self.tag_name)?;

        for (key, value) in &self.attributes {
            match (key.as_str(), value.as_str()) {
                ("", _) => continue,
                (key, "") => write!(f, " {}", key)?,
                (key, value) => write!(f, " {}={}", key, value)?,
            }
        }
        write!(f, ">")?;

        for child in &self.content {
            write!(f, "{}", child.to_string())?;
        }

        write!(f, "</{}>", self.tag_name)?;
        return Ok(());
    }
}
