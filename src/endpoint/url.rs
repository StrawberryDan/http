use std::borrow::Borrow;
use crate::Error;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone)]
pub enum Segment {
    Static(String),
    Dynamic(String)
}

impl ToString for Segment {
    fn to_string(&self) -> String {
        match self {
            Segment::Static(s) | Segment::Dynamic(s) => s.clone(),
        }
    }
}

impl TryFrom<&str> for Segment {
    type Error = Error;

    fn try_from(txt: &str) -> Result<Self, Self::Error> {
        if txt.chars().any(|c| !c.is_ascii()) {
            return Err(Error::InvalidEndpoint);
        }

        if txt.starts_with("<") && txt.ends_with(">") {
            let stripped = &txt[1..txt.len() - 1];
            Ok(Segment::Dynamic(stripped.to_owned()))
        } else {
            Ok(Segment::Static(txt.to_owned()))
        }
    }
}

impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Segment::Static(a), Segment::Static(b)) => a == b,
            (Segment::Dynamic(_), Segment::Dynamic(_)) => true,
            (_, _) => false,
        }
    }
}

pub struct URL {
    segments: Vec<Segment>,
}

impl URL {
    pub fn from_string(string: impl Borrow<str>) -> Result<Self, Error> {
        let string = string.borrow();
        let mut segments: Vec<_> = string.split("/").skip(1).map(|s| Segment::try_from(s)).collect();
        if segments.iter().any(|s| s.is_err()) {
            return Err(Error::URLParse);
        }

        Ok(
            Self {
                segments: segments.into_iter().map(|s| s.unwrap()).collect()
            }
        )
    }

    pub fn segments(&self) -> Vec<&Segment> {
        self.segments.iter().collect::<Vec<_>>()
    }
}

impl Display for URL {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Debug for URL {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.segments.iter().map(|x| x.to_string()).collect::<Vec<_>>())
    }
}


