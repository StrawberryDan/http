mod parse;

use crate::http::{Method, Request, Response};
use std::fmt::{Display, Formatter};

pub use parse::Bindings;
pub use parse::EndpointTable;

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

pub trait EndpointFunction {
    fn handle(&self, request: Request, bindings: Bindings) -> Option<Response>;
}