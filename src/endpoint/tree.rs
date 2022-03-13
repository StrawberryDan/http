use std::collections::HashMap;
use super::*;
use super::url::{Segment as URLSegment, URL};
use crate::Error;
use crate::http::RequestCallback;
use bit_vec::BitVec;

pub struct Tree {
    root: Vec<(URLSegment, Node)>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            root: Vec::new(),
        }
    }

    pub fn add(&mut self, mut endpoint: Endpoint, callback: RequestCallback) -> Result<(), Error> {
        let (root, stem, leaf) = match endpoint.resource.len() {
            0 => panic!("Attempt to add endpoint to tree with no resource specified"),
            1 => (endpoint.resource.remove(0), None, None),
            2 => {
                let root = endpoint.resource.remove(0);
                let leaf = endpoint.resource.remove(0);
                (root, None, Some(leaf))
            },
            n => {
                let root = endpoint.resource.remove(0);
                let leaf = Some(endpoint.resource.remove(n - 2));
                (root, Some(endpoint.resource), leaf)
            },
        };

        if !self.root.iter().any(|n| n.0 == root) {
            self.root.push((root.clone(), Node::new()));
        } else if matches!((&stem, &leaf), (None, None)) {
            return Err(Error::DuplicateEndpoint);
        }

        let mut cursor = &mut self.root.iter_mut().find(|n| n.0 == root).unwrap().1;
        for seg in stem.unwrap_or(Vec::new()) {
            if !cursor.children.iter().any(|s| s.0 == seg) {
                cursor.children.push((seg.clone(), Node::new()));
            }

            cursor = &mut cursor.children.iter_mut().find(|s| s.0 == seg).unwrap().1;
        }

        if let Some(leaf) = leaf {
            if cursor.children.iter().any(|s| s.0 == leaf) {
                return Err(Error::DuplicateEndpoint);
            } else {
                cursor.children.push((leaf.clone(), Node::new()));
                cursor = &mut cursor.children.iter_mut().find(|s| s.0 == leaf).unwrap().1;
            }
        }

        cursor.value = Some(callback);

        Ok(())
    }

    pub fn find_match(&self, url: &URL) -> Option<(&RequestCallback, Bindings)> {
        let cursor = &self.root;
        let mut candidates: Vec<_> = cursor.iter().map(|x| (x, Bindings::new(), BitVec::new())).collect();
        let segments = url.segments();
        if segments.is_empty() {  return None; }
        let (leaf, stem) = segments.split_last().unwrap();
        for seg in stem{
            candidates = candidates.into_iter()
                // Add binding success and new binding table
                .map(|(node, bindings, mut priority)| {
                    let (b, b2) = bind(&node.0, &seg.to_string(), &bindings);
                    if let URLSegment::Static(_) = &node.0 {
                        priority.push(true);
                    } else {
                        priority.push(false);
                    }
                    (b, node, b2, priority)
                })
                // Remove failed bindings and remove flag
                .filter(|(b, _, _, _)| *b).map(|(_, node, bindings, priority)| (node, bindings, priority))
                // Expand children and flatten
                .map(|(node, bindings, priority)| node.1.children.iter().map(move |node| (node.clone(), bindings.clone(), priority.clone())))
                .flatten()
                .collect();
        }
        // Bind final segment without expanding children
        candidates = candidates.into_iter()
            .map(|(node, bindings, mut priority)| {
                let (b, b2) = bind(&node.0, &leaf.to_string(), &bindings);
                if let URLSegment::Static(_) = &node.0 {
                    priority.push(true);
                } else {
                    priority.push(false);
                }
                (b, node, b2, priority)
            })
            .filter(|(b, _, _, _)| *b)
            .map(|(_, node, bindings, priority)| (node, bindings, priority))
            .collect();

        if candidates.is_empty() {
            None
        } else {
            candidates.sort_by(|(_, _, a), (_, _, b)| a.cmp(b).reverse());
            let callback = candidates[0].0.1.value.as_ref().unwrap();
            let bindings = candidates[0].1.clone();
            return Some((callback, bindings));
        }
    }
}

fn bind(seg: &URLSegment, val: &str, bindings: &Bindings) -> (bool, Bindings) {
    match seg {
        URLSegment::Static(seg) => (seg == val, bindings.clone()),
        URLSegment::Dynamic(seg) => {
            let mut bindings = bindings.clone();
            bindings.insert(seg.clone(), val.to_string());
            (true, bindings)
        }
    }
}

struct Node {
    value: Option<RequestCallback>,
    children: Vec<(URLSegment, Node)>,
}

impl Node {
    fn new() -> Self {
        Self {
            value: None,
            children: Vec::new(),
        }
    }
}

pub type Bindings = HashMap<String, String>;