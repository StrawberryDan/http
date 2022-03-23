mod parse;

use crate::http::{Method, Request, Response};
use std::fmt::{Display, Formatter};

pub use parse::Bindings;
pub use parse::EndpointTable;

pub type Callback = fn(&Request, &Bindings) -> Option<Response>;

pub struct Endpoint {
    method: Method,
    resource: String,
}

impl Endpoint {
    pub fn new(method: Method, resource: &str) -> Self {
        Endpoint {
            method,
            resource: resource.to_string(),
        }
    }

    pub fn verb(&self) -> Method {
        self.method
    }

    pub fn resource(&self) -> &String {
        &self.resource
    }
}

#[macro_export]
macro_rules! endpoint {
    ($v: ident, $r: literal) => {
        crate::http::endpoint::Endpoint::new(crate::http::Method::$v, $r)
    };
}

#[allow(unused)]
pub(crate) use endpoint;
