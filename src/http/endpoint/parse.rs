use std::collections::HashMap;
use super::*;
use crate::{Error, url::URL};
use bit_vec::BitVec;

#[derive(Clone)]
enum Segment {
    Constant(String),
    Variable(String)
}

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::Constant(s) | Segment::Variable(s) => write!(f, "{}", s),
        }
    }
}

fn to_segments(str: &str) -> Vec<Segment> {
    str.split("/").skip(1).map(
        |s| if s.starts_with("<") && s.ends_with(">") {
            Segment::Variable(s[1..s.len() - 1].to_string())
        } else {
            Segment::Constant(s.to_string())
        }
    ).collect()
}

pub struct ParseTree {
    root: Vec<(Segment, Node)>,
}

impl ParseTree {
    pub fn new() -> Self {
        Self {
            root: Vec::new(),
        }
    }

    pub fn add(&mut self, endpoint: Endpoint, callback: Callback) -> Result<(), Error> {
        let mut segments = to_segments(endpoint.resource());
        // Split into Root, Stem and Leaf
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

        // Check if root node already exists
        if !self.root.iter().any(|n| matches(&n.0, &root)) {
            self.root.push((root.clone(), Node::new()));
        } else if matches!((&stem, &leaf), (None, None)) {
            // Report duplication if this the end of the endpoint
            return Err(Error::DuplicateEndpoint);
        }

        // Fill in stem nodes
        let mut cursor = &mut self.root.iter_mut().find(|n| matches(&n.0, &root)).unwrap().1;
        for seg in stem.unwrap_or(Vec::new()) {
            if !cursor.children.iter().any(|s| matches(&s.0, &seg)) {
                cursor.children.push((seg.clone(), Node::new()));
            }

            cursor = &mut cursor.children.iter_mut().find(|s| matches(&s.0, &seg)).unwrap().1;
        }

        // If there is a leaf
        if let Some(leaf) = leaf {
            // Error if node already exists
            if cursor.children.iter().any(|s| matches(&s.0, &leaf)) {
                return Err(Error::DuplicateEndpoint);
            } else {
                cursor.children.push((leaf.clone(), Node::new()));
                cursor = &mut cursor.children.iter_mut().find(|s| matches(&s.0, &leaf)).unwrap().1;
            }
        }

        // Add callback to added node
        cursor.value = Some(callback);

        // Great Success
        Ok(())
    }

    pub fn find_match(&self, url: &URL) -> Option<(Callback, Bindings)> {
        // Get all root nodes and mark with empty bindings and 0 priority to be filled in
        let mut candidates: Vec<_> = self.root.iter().map(|x| (x, Bindings::new(), BitVec::new())).collect();
        // Split the url resource into segments
        let segments: Vec<_> = url.resource().split("/").map(|s| Segment::Constant(s.to_string())).collect();

        if segments.is_empty() {  return None; }
        let (leaf, stem) = segments.split_last().unwrap();
        for seg in stem{
            candidates = bind_and_filter(candidates.into_iter(), &seg)
                // Expand children and flatten and collect
                .map(|(node, bindings, priority)| node.1.children.iter()
                .map(move |node| (node, bindings.clone(), priority.clone())))
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
        .filter(|(b, _, _, _)| *b)
        .map(|(_, node, bindings, priority)| (node, bindings, priority))
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

fn matches(a: &Segment, b: &Segment) -> bool {
    use Segment::*;
    match (a, b) {
        (Constant(a), Constant(b)) => a == b,
        (Variable(_), Variable(_)) => true,
        _ => false,
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