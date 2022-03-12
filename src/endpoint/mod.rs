mod tree;

use std::convert::TryFrom;

use crate::Error;
use crate::http::Verb as HTTPVerb;

pub use tree::Tree;
pub use tree::Bindings as URLBindings;

#[derive(Clone)]
pub enum URLSegment {
    Static(String),
    Dynamic(String)
}

impl ToString for URLSegment {
    fn to_string(&self) -> String {
        match self {
            URLSegment::Static(s) | URLSegment::Dynamic(s) => s.clone(),
        }
    }
}

impl TryFrom<&str> for URLSegment {
    type Error = Error;

    fn try_from(txt: &str) -> Result<Self, Self::Error> {
        if txt.chars().any(|c| !c.is_ascii()) {
            return Err(Error::InvalidEndpoint);
        }

        if txt.starts_with("<") && txt.ends_with(">") {
            let stripped = &txt[1..txt.len() - 1];
            Ok(URLSegment::Dynamic(stripped.to_owned()))
        } else {
            Ok(URLSegment::Static(txt.to_owned()))
        }
    }
}

impl PartialEq for URLSegment {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (URLSegment::Static(a), URLSegment::Static(b)) => a == b,
            (URLSegment::Dynamic(_), URLSegment::Dynamic(_)) => true,
            (_, _) => false,
        }
    }
}

pub struct Endpoint {
    verb: HTTPVerb,
    resource: Vec<URLSegment>,
}

impl Endpoint {
    pub fn new(verb: HTTPVerb, resource: &str) -> Result<Self, Error> {
        let segments: Vec<_> = resource.split("/")
            .map(|f| URLSegment::try_from(f))
            .skip(1)
            .collect();

        if segments.iter().any(|s| s.is_err()) {
            Err( Error::InvalidEndpoint )
        } else {
            Ok( Endpoint{ verb, resource: segments.into_iter().map(|s| s.unwrap()).collect() } )
        }
    }

    pub fn verb(&self) -> HTTPVerb {
        self.verb
    }
}

#[macro_export]
macro_rules! new_endpoint {
    ($v: ident, $r: literal) => { crate::endpoint::Endpoint::new(crate::http::Verb::$v, $r).unwrap()};
}

pub(crate) use new_endpoint;