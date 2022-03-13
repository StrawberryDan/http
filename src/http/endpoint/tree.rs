use std::collections::HashMap;
use super::*;
use crate::http::endpoint::Segment;
use crate::{Error, url::URL};
use bit_vec::BitVec;

pub struct Tree {
    root: Vec<(Segment, Node)>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            root: Vec::new(),
        }
    }

    pub fn add(&mut self, endpoint: Endpoint, callback: Callback) -> Result<(), Error> {
        let mut segments = endpoint.segments();
        let (root, stem, leaf) = match segments.len() {
            0 => panic!("Attempt to add endpoint to tree with no resource specified"),
            1 => (segments.remove(0), None, None),
            2 => {
                let root = segments.remove(0);
                let leaf = segments.remove(0);
                (root, None, Some(leaf))
            },
            n => {
                let root = segments.remove(0);
                let leaf = Some(segments.remove(n - 2));
                (root, Some(segments), leaf)
            },
        };

        if !self.root.iter().any(|n| n.0.matches(&root)) {
            self.root.push((root.clone(), Node::new()));
        } else if matches!((&stem, &leaf), (None, None)) {
            return Err(Error::DuplicateEndpoint);
        }

        let mut cursor = &mut self.root.iter_mut().find(|n| n.0.matches(&root)).unwrap().1;
        for seg in stem.unwrap_or(Vec::new()) {
            if !cursor.children.iter().any(|s| s.0.matches(&seg)) {
                cursor.children.push((seg.clone(), Node::new()));
            }

            cursor = &mut cursor.children.iter_mut().find(|s| s.0.matches(&seg)).unwrap().1;
        }

        if let Some(leaf) = leaf {
            if cursor.children.iter().any(|s| s.0.matches(&leaf)) {
                return Err(Error::DuplicateEndpoint);
            } else {
                cursor.children.push((leaf.clone(), Node::new()));
                cursor = &mut cursor.children.iter_mut().find(|s| s.0.matches(&leaf)).unwrap().1;
            }
        }

        cursor.value = Some(callback);

        Ok(())
    }

    pub fn find_match(&self, url: &URL) -> Option<(Callback, Bindings)> {
        let mut candidates: Vec<_> = self.root.iter().map(|x| (x, Bindings::new(), BitVec::new())).collect();
        let segments: Vec<_> = url.resource_split().into_iter().map(|s| Segment::Constant(s)).collect();
        if segments.is_empty() {  return None; }
        let (leaf, stem) = segments.split_last().unwrap();
        for seg in stem{
            candidates = bind_and_filter(candidates.into_iter(), &seg)
                // Expand children and flatten
                .map(|(node, bindings, priority)| node.1.children.iter().map(move |node| (node, bindings.clone(), priority.clone())))
                .flatten()
                .collect();
        }
        // Bind final segment without expanding children
        candidates = bind_and_filter(candidates.into_iter(), &leaf)
            .collect();

        if candidates.is_empty() {
            None
        } else {
            candidates.sort_by(|(_, _, a), (_, _, b)| a.cmp(b).reverse());
            let callback = candidates[0].0.1.value.unwrap();
            let bindings = candidates[0].1.clone();
            return Some((callback, bindings));
        }
    }
}

fn bind_and_filter<'r>(input: impl Iterator<Item = (&'r (Segment, Node), Bindings, BitVec)> + 'r, seg: &'r Segment) -> impl Iterator<Item = (&'r (Segment, Node), Bindings, BitVec)> + 'r {
    // Add binding success and new binding table
    input.map(move |(node, bindings, mut priority)| {
        let (b, b2) = bind(&node.0, &seg.to_string(), &bindings);
        if let Segment::Constant(_) = &node.0 {
            priority.push(true);
        } else {
            priority.push(false);
        }
        (b, node, b2, priority)
    })
    // Remove failed bindings and remove flag
    .filter(|(b, _, _, _)| *b).map(|(_, node, bindings, priority)| (node, bindings, priority))
}

fn bind(seg: &Segment, val: &str, bindings: &Bindings) -> (bool, Bindings) {
    match seg {
        Segment::Constant(seg) => (seg == val, bindings.clone()),
        Segment::Variable(seg) => {
            let mut bindings = bindings.clone();
            bindings.insert(seg.clone(), val.to_string());
            (true, bindings)
        }
    }
}

struct Node {
    value: Option<Callback>,
    children: Vec<(Segment, Node)>,
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