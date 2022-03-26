use std::borrow::Borrow;

#[derive(Debug, Clone)]
pub struct Header {
    data: Vec<HeaderEntry>,
}

impl Header {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }

    pub fn get_first(&self, key: impl Borrow<str>) -> Option<&str> {
        self.data.iter()
            .find(|h| &h.key == key.borrow())
            .map(|h| h.value.as_str())
    }

    pub fn get_all(&self, key: impl Borrow<str>) -> Vec<&str> {
        self.data.iter()
            .filter(|h| &h.key == key.borrow())
            .map(|h| h.value.as_str())
            .collect()
    }

    pub fn add(&mut self, key: impl Borrow<str>, value: impl Borrow<str>) {
        let entry = HeaderEntry { key: key.borrow().to_owned(), value: value.borrow().to_owned() };

        match self.data.binary_search(&entry) {
            Ok(_) => (),
            Err(pos) => self.data.insert(pos, entry),
        }
    }

    pub fn replace(&mut self, key: impl Borrow<str>, value: impl Borrow<str>) {
        self.remove(key.borrow());
        self.add(key, value);
    }

    pub fn remove(&mut self, key: impl Borrow<str>) {
        self.data.retain(|h| h.key != key.borrow())
    }

    pub fn cookie(&self, name: impl Borrow<str>) -> Option<&str> {
        self.get_all("Cookie")
            .into_iter()
            .map(|s| s.split_once('='))
            .filter(|x| x.is_some())
            .map(|x| unsafe { x.unwrap_unchecked() })
            .filter(|(key, _)| *key == name.borrow())
            .map(|(_, val)| val)
            .nth(0)
    }
}

impl IntoIterator for Header {
    type Item = (String, String);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter().map(|h| (h.key, h.value)).collect::<Vec<_>>().into_iter()
    }
}

impl<'r> IntoIterator for &'r Header {
    type Item = (&'r str, &'r str);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
            .map(|h| (h.key.as_str(), h.value.as_str()))
            .collect::<Vec<_>>().into_iter()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct HeaderEntry {
    key: String,
    value: String,
}