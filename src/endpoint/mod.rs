mod tree;
mod url;

use std::convert::TryFrom;

use crate::Error;
use crate::http::Verb as HTTPVerb;

pub use tree::Tree;
pub use tree::Bindings as URLBindings;
pub use url::{URL, Segment as URLSegment};

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

#[allow(unused)]
pub(crate) use new_endpoint;