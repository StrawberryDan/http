use std::collections::{HashMap};

use super::*;
use crate::url::URL;

pub struct EndpointTable {
    endpoints: Vec<(EndpointURI, Box<dyn EndpointFunction + Send + Sync>)>,
}

impl EndpointTable {
    pub fn new() -> Self {
        Self { endpoints: Vec::new() }
    }

    pub fn add(&mut self, endpoint: Endpoint, handler: Box<dyn EndpointFunction + Send + Sync>) {
        self.endpoints.push((EndpointURI::from(&endpoint), handler));
    }

    pub fn find_match(&self, method: Method, url: &URL) -> Option<(&Box<dyn EndpointFunction + Send + Sync>, Bindings)> {
        let segments: Vec<_> = url.resource().iter().map(|s| Segment::Constant(s.to_string())).collect();

        let candidates: Vec<_> = self.endpoints.iter()
            .filter(|(e, _)| e.method == method)
            .filter(|(e, _)| e.segments.len() == segments.len())
            .map(|(e, h)| (e, h, e.try_match(&segments)))
            .filter(|(_, _, bindings)| bindings.is_some())
            .collect();

        if candidates.len() == 0 {
            return None;
        } else if candidates.len() > 1 {
            eprintln!("Ambiguous endpoint match! Possible to match {:?} {} to the following endpoints: {:?}",
                      method, url.as_string().unwrap(),
                      candidates.iter()
                          .map(|(e, _, _)|
                              e.segments.iter().map(
                                  |s| match s {
                                      Segment::Constant(s) => s.clone(),
                                      Segment::Variable(s) => format!("<{}>", s),
                                  }
                              )
                                  .fold(String::from(""), |a, b| format!("{}/{}", a, b))
                          )
                          .collect::<Vec<_>>());
        }

        return Some((candidates[0].1, candidates[0].2.clone().unwrap()));
    }
}

#[derive(Debug)]
struct EndpointURI {
    method: Method,
    segments: Vec<Segment>,
}

impl EndpointURI {
    pub fn from(endpoint: &Endpoint) -> Self {
        Self {
            method: endpoint.method,
            segments: to_segments(endpoint.resource()),
        }
    }

    pub fn try_match(&self, other: &Vec<Segment>) -> Option<Bindings> {
        let mut bindings = Bindings::new();

        for (a, b) in self.segments.iter().zip(other.iter()) {
            match Segment::try_match(a, b, &mut bindings) {
                false => return None,
                true => (),
            }
        }

        return Some(bindings);
    }
}

pub type Bindings = HashMap<String, String>;

#[derive(Debug)]
enum Segment {
    Constant(String),
    Variable(String),
}

impl Segment {
    pub fn try_match(a: &Segment, b: &Segment, bindings: &mut Bindings) -> bool {
        use Segment::*;

        match (a, b) {
            (Constant(a), Constant(b)) => a == b,
            (Variable(_), Variable(_)) => true,
            (Variable(v), Constant(s)) | (Constant(s), Variable(v)) => {
                bindings.insert(v.clone(), s.clone());
                true
            }
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

fn to_segments(str: &str) -> Vec<Segment> {
    str.split("/")
        .skip(1)
        .map(|s| {
            if s.starts_with("<") && s.ends_with(">") {
                Segment::Variable(s[1..s.len() - 1].to_string())
            } else {
                Segment::Constant(s.to_string())
            }
        })
        .collect()
}
