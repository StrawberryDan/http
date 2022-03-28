use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::html::element::Element;
use crate::html::tag::Tag;

impl FromStr for Element {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = tokenise(s)?;
        let lexemes = lex(tokens)?;
        let lexemes = fix_script_tags(lexemes)?;
        let levelled = tag_depths(lexemes)?;
        let tree = parse_tree(levelled).remove(0);
        parse_element(tree)
    }
}

fn tokenise(s: &str) -> Result<Vec<Token>, ()> {
    use Token::*;

    let mut tokens = Vec::new();
    let mut next: Option<Token> = None;
    let mut inside_tag = false;

    for c in s.chars() {
        match (&next, c) {
            (None, '<') => {
                next = Some(OpenTag);
                inside_tag = true;
            }

            (None, '>') => {
                next = Some(CloseTag);
                inside_tag = false;
            }

            (None, c) if c.is_whitespace() => continue,

            (None, c) if !c.is_whitespace() => {
                next = Some(Word(String::from(c)));
            }

            (Some(Equals), '"') if !c.is_whitespace() => {
                tokens.push(next.take().unwrap());
                next = Some(Quote(String::from(c)));
            }

            (Some(Equals), c) if c.is_whitespace() => continue,

            (Some(Equals), c) if !c.is_whitespace() => {
                tokens.push(next.take().unwrap());
                next = Some(Word(String::from(c)));
            }

            (Some(CloseTag), '<') => {
                tokens.push(next.take().unwrap());
                next = Some(OpenTag);
                inside_tag = true;
            }

            (Some(CloseTag), c) if c.is_whitespace() => {
                continue;
            }

            (Some(CloseTag), c) => {
                tokens.push(next.take().unwrap());
                next = Some(Word(String::from(c)));
            }

            (Some(_), '>') => {
                tokens.push(next.take().unwrap());
                next = Some(CloseTag);
                inside_tag = false;
            }

            (Some(OpenTag), '/') => {
                tokens.push(next.take().unwrap());
                next = Some(ClosingMark);
            }

            (Some(OpenTag), c) if c.is_whitespace() => {
                continue;
            }

            (Some(OpenTag), c) => {
                tokens.push(next.take().unwrap());
                next = Some(Word(String::from(c)));
            }

            (Some(Word(_)), '=') => {
                tokens.push(next.take().unwrap());
                next = Some(Equals);
            }

            (Some(Word(_)), '<') => {
                tokens.push(next.take().unwrap());
                next = Some(OpenTag);
                inside_tag = true;
            }

            (Some(Word(_)), c) if !c.is_whitespace() && inside_tag => {
                match next.as_mut().unwrap() {
                    Word(s) => s.push(c),
                    _ => unreachable!(),
                }
            }

            (Some(Word(_)), c) if c.is_whitespace() && inside_tag => {
                tokens.push(next.take().unwrap());
            }

            (Some(Word(_)), c) if !inside_tag => {
                match next.as_mut().unwrap() {
                    Word(s) => s.push(c),
                    _ => unreachable!(),
                }
            }

            (Some(Quote(_)), '"') => {
                match next.as_mut() {
                    Some(Quote(s)) => {
                        s.push(c);
                        tokens.push(next.take().unwrap());
                    }
                    _ => unreachable!(),
                }
            }

            (Some(Quote(_)), c) => {
                match next.as_mut() {
                    Some(Quote(s)) => s.push(c),
                    _ => unreachable!(),
                }
            }

            (Some(ClosingMark), c) if c.is_whitespace() => {
                continue;
            }

            (Some(ClosingMark), c) => {
                tokens.push(next.take().unwrap());
                next = Some(Word(String::from(c)));
            }

            _ => return Err(()),
        }
    }

    if let Some(t) = next.take() {
        tokens.push(t);
    }

    return Ok(tokens);
}

#[derive(Debug, Eq, PartialEq)]
enum Token {
    OpenTag,
    CloseTag,
    Equals,
    ClosingMark,
    Word(String),
    Quote(String),
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::OpenTag => write!(f, "<"),
            Token::CloseTag => write!(f, ">"),
            Token::Equals => write!(f, "="),
            Token::ClosingMark => write!(f, "/"),
            Token::Word(s) => write!(f, "{}", s),
            Token::Quote(s) => write!(f, "{}", s),
        }
    }
}

fn lex(tokens: Vec<Token>) -> Result<Vec<Lexeme>, ()> {
    use Token::*;
    use Lexeme::*;

    let mut lexemes = Vec::new();
    let mut next: Option<Lexeme> = None;

    for token in tokens {
        match (&next, &token) {
            (None, OpenTag) => {
                next = Some(OpeningTag(vec![token]));
            }

            (None, _) => {
                next = Some(Text(vec![token]));
            }

            (Some(OpeningTag(_)), CloseTag) => {
                match &mut next {
                    Some(OpeningTag(v)) => {
                        v.push(token);
                        lexemes.push(next.take().unwrap());
                    }
                    _ => unreachable!(),
                }
            }

            (Some(OpeningTag(v)), ClosingMark) if v.len() == 1 => {
                if matches!(v[0], OpenTag) {
                    let temp = next.take().unwrap();
                    if let OpeningTag(v) = temp {
                        next = Some(ClosingTag(v));
                    } else {
                        unreachable!()
                    }
                } else {
                    continue;
                }
            }

            (Some(OpeningTag(_)), _) => {
                match &mut next {
                    Some(OpeningTag(v)) => v.push(token),
                    _ => unreachable!(),
                }
            }

            (Some(ClosingTag(_)), CloseTag) => {
                match &mut next {
                    Some(ClosingTag(v)) => v.push(token),
                    _ => unreachable!(),
                }
                lexemes.push(next.take().unwrap());
            }

            (Some(ClosingTag(_)), _) => {
                match &mut next {
                    Some(ClosingTag(v)) => v.push(token),
                    _ => unreachable!(),
                }
            }

            (Some(Text(_)), OpenTag) => {
                lexemes.push(next.take().unwrap());
                next = Some(OpeningTag(vec![token]));
            }

            (Some(Text(_)), _) => {
                match &mut next {
                    Some(Text(v)) => v.push(token),
                    _ => unreachable!(),
                }
            }
        }
    }

    return Ok(lexemes);
}

#[derive(Debug)]
enum Lexeme {
    OpeningTag(Vec<Token>),
    ClosingTag(Vec<Token>),
    Text(Vec<Token>),
}

impl Display for Lexeme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Lexeme::OpeningTag(v) | Lexeme::ClosingTag(v) | Lexeme::Text(v) => {
                write!(f, "{}", v.iter().map(|x| x.to_string()).reduce(|a, b| format!("{}{}", a, b)).unwrap())
            }
        }
    }
}

fn fix_script_tags(mut lexemes: Vec<Lexeme>) -> Result<Vec<Lexeme>, ()> {
    let opening_tags: Vec<_> = lexemes.iter().enumerate().filter(|l| if let Lexeme::OpeningTag(_) = l.1 { true } else { false }).collect();
    let closing_tags: Vec<_> = lexemes.iter().enumerate().filter(|l| if let Lexeme::ClosingTag(_) = l.1 { true } else { false }).collect();

    let script_opens: Vec<_> = opening_tags.iter().filter(|l| match l.1 {
        Lexeme::OpeningTag(t) => {
            if let Token::Word(s) = &t[1] {
                return s == "script";
            } else {
                unreachable!()
            }
        }
        _ => unreachable!(),
    }).collect();

    let script_closes: Vec<_> = closing_tags.iter().filter(|l| match l.1 {
        Lexeme::ClosingTag(t) => {
            if let Token::Word(s) = &t[1] {
                return s == "script";
            } else {
                unreachable!()
            }
        }
        _ => unreachable!(),
    }).collect();

    if script_opens.len() != script_closes.len() {
        return Err(());
    }

    let script_pairs: Vec<_> = script_opens.into_iter().map(|x| x.0).zip(script_closes.into_iter().map(|x| x.0)).rev().collect();

    for (start, end) in script_pairs {
        let mut inner = Vec::new();
        for _ in start + 1 .. end {
            inner.push(lexemes.remove(start + 1));
        }

        let inner = inner.into_iter().map(|x| x.to_string()).reduce(|a, b| format!("{}{}", a, b)).unwrap();
        lexemes.insert(start + 1, Lexeme::Text(vec![Token::Word(inner)]));
    }

    return Ok(lexemes);
}

fn tag_depths(lexemes: Vec<Lexeme>) -> Result<Vec<(Lexeme, isize)>, ()> {
    let mut x: isize = 0;

    let marked = lexemes.into_iter().map(
        |lexeme| {
            match lexeme {
                Lexeme::OpeningTag(_) => {
                    x += 1;
                    (lexeme, x - 1)
                }

                Lexeme::ClosingTag(_) => {
                    x -= 1;
                    (lexeme, x)
                }

                Lexeme::Text(_) => {
                    (lexeme, x)
                }
            }
        }
    ).collect::<Vec<_>>();

    if marked.iter().any(|(_, x)| *x < 0) {
        return Err(());
    }

    return Ok(marked);
}

#[derive(Debug)]
enum Tree {
    Stem(Lexeme, Vec<Tree>, Lexeme),
    Leaf(Lexeme),
}

fn parse_tree(mut lexemes: Vec<(Lexeme, isize)>) -> Vec<Tree> {
    let mut result = Vec::new();

    while !lexemes.is_empty() {
        while matches!(&lexemes[0], (Lexeme::Text(_), _)) {
            result.push(Tree::Leaf(lexemes.remove(0).0));
        }

        let start = lexemes.remove(0);
        let end_idx = (0..lexemes.len()).find(|i| lexemes[*i].1 == start.1).unwrap();
        let end = lexemes.remove(end_idx);

        let mut middle = {
            let mut mid = Vec::new();
            for _ in 0..end_idx { mid.push(lexemes.remove(0)) }
            mid
        };

        if middle.len() == 0 {
            result.push(Tree::Stem(start.0, vec![], end.0));
        } else if middle.len() == 1 {
            result.push(Tree::Stem(start.0, vec![Tree::Leaf(middle.remove(0).0)], end.0));
        } else {
            result.push(Tree::Stem(start.0, parse_tree(middle), end.0));
        }
    }

    return result;
}

fn parse_element(tree: Tree) -> Result<Element, ()> {
    match tree {
        Tree::Stem(start, content, end) => {
            let mut tag = parse_tag(start, end);
            let content: Vec<_> = content.into_iter().map(|x| parse_element(x).unwrap()).collect();
            for x in content {
                match x {
                    Element::Text(x) => tag = tag.with_text(&x),
                    Element::Tag(x) => tag = tag.with_child(x),
                }
            }
            return Ok(Element::Tag(tag));
        }

        Tree::Leaf(x) => {
            match x {
                Lexeme::OpeningTag(_) | Lexeme::ClosingTag(_) => unreachable!(),
                Lexeme::Text(t) => return Ok(
                    Element::Text(
                        t.into_iter()
                            .map(|t| t.to_string())
                            .reduce(|a, b| format!("{}{}", a, b)).unwrap_or(String::new())
                    )
                )
            }
        }
    }
}

fn parse_tag(start: Lexeme, end: Lexeme) -> Tag {
    let (start, end) = match (start, end) {
        (Lexeme::OpeningTag(a), Lexeme::ClosingTag(b)) => (a, b),
        _ => unreachable!(),
    };

    assert_eq!(start[1], end[1]);

    let mut tag = Tag::new(&start[1].to_string());

    let mut i = 2;
    while i < start.len() - 1 {
        if let Token::Word(key) = &start[i] {
            if let Token::Equals = &start[i + 1] {
                if let Token::Word(value) = &start[i + 2] {
                    tag = tag.with_attribute(key, value);
                } else if let Token::Quote(value) = &start[i + 2] {
                    tag = tag.with_attribute(key, &value.strip_prefix('"').unwrap().strip_suffix('"').unwrap().to_string())
                }
                i += 3;
            } else {
                tag = tag.with_attribute(key, &String::new());
                i += 1;
            }
        }
    }

    return tag;
}