use std::collections::HashMap;
use super::*;
use crate::Error;
use crate::server::Callback;

pub struct Tree {
    root: Vec<(URLSegment, Node)>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            root: Vec::new(),
        }
    }

    pub fn add(&mut self, mut endpoint: Endpoint, callback: Callback) -> Result<(), Error> {
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

    pub fn find_match(&self, url: &String) -> Option<(&Callback, Bindings)> {
        let url: Vec<_> = url.split("/").skip(1).collect();

        let mut cursor = &self.root;
        let mut candidates: Vec<_> = cursor.iter().map(|x| (x, Bindings::new())).collect();
        for seg in &url[..url.len() - 1] {
            candidates = candidates.into_iter()
                // Add binding success and new binding table
                .map(|(node, bindings)| {
                    let (b, b2) = bind(&node.0, seg, &bindings);
                    (b, node, b2)
                })
                // Remove failed bindings and remove flag
                .filter(|(b, _, _)| *b).map(|(_, node, bindings)| (node, bindings))
                // Expand children and flatten
                .map(|(node, bindings)| node.1.children.iter().map(move |node| (node.clone(), bindings.clone())))
                .flatten()
                .collect();
        }
        // Bind final segment without expanding children
        candidates = candidates.into_iter()
            .map(|(node, bindings)| {
                let (b, b2) = bind(&node.0, url.last().unwrap(), &bindings);
                (b, node, b2)
            })
            .filter(|(b, _, _)| *b)
            .map(|(_, node, bindings)| (node, bindings))
            .collect();

        if candidates.is_empty() {
            None
        } else {
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
    value: Option<Callback>,
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