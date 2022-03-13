use std::borrow::Borrow;
use crate::Error;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use crate::endpoint::URLSegment;

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

#[derive(Debug)]
pub struct URL {
    protocol: String,
    username: String,
    password: String,
    host: String,
    resource: String,
}

impl URL {
    pub fn from_string(string: impl Borrow<str>) -> Result<Self, Error> {
        let string = string.borrow();

        let (protocol, rest) = string.split_once("://").unwrap_or(("", string));
        let (host, resource) = rest.split_once("/").unwrap_or(("", rest));

        let (user, host) = host.split_once("@").unwrap_or(("", host));
        let (username, password) = user.split_once(":").unwrap_or(("", ""));

        Ok(
            Self {
                protocol: protocol.to_string(),
                username: username.to_string(),
                password: password.to_string(),
                host: host.to_string(),
                resource: format!("/{}", resource.trim()),
            }
        )
    }

    pub fn resource(&self) -> &String {
        &self.resource
    }

    pub fn segments(&self) -> Vec<Segment> {
        self.resource.split("/").map(|x| URLSegment::try_from(x).unwrap()).collect::<Vec<_>>()
    }
}

impl Display for URL {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://{}{}", self.protocol, self.host, self.resource)
    }
}


