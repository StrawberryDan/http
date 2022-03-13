mod tree;

use std::fmt::{Display, Formatter};
use crate::http::{Method, Request, Response};

pub use tree::Tree;
pub use tree::Bindings;

pub type Callback = fn(&Request, &Bindings) -> Option<Response>;

pub struct Endpoint {
    method: Method,
    resource: String,
}

impl Endpoint {
    pub fn new(method: Method, resource: &str) -> Self {
        Endpoint {
            method,
            resource: resource.to_string()
        }
    }

    pub fn verb(&self) -> Method {
        self.method
    }

    fn segments(&self) -> Vec<Segment> {
        self.resource.split("/").skip(1).map(
            |s| if s.starts_with("<") && s.ends_with(">") {
                Segment::Variable(s[1..s.len() - 1].to_string())
            } else {
                Segment::Constant(s.to_string())
            }
        ).collect()
    }
}

#[derive(Clone)]
enum Segment {
    Constant(String),
    Variable(String)
}

impl Segment {
    fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Segment::Constant(a), Segment::Constant(b)) => a == b,
            (Segment::Variable(_), Segment::Variable(_)) => true,
            _ => false,
        }
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::Constant(s) | Segment::Variable(s) => write!(f, "{}", s),
        }
    }
}

#[macro_export]
macro_rules! endpoint {
    ($v: ident, $r: literal) => { crate::http::endpoint::Endpoint::new(crate::http::Method::$v, $r)};
}

#[allow(unused)]
pub(crate) use endpoint;