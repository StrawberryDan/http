use std::borrow::Borrow;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct URL {
    protocol: Option<String>,
    username: Option<String>,
    password: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    resource: Vec<String>,
    parameters: HashMap<String, String>,
}

impl URL {
    pub fn new() -> Self {
        Self {
            protocol: None,
            username: None,
            password: None,
            host: None,
            port: None,
            resource: vec!["/".to_string()],
            parameters: HashMap::new(),
        }
    }

    pub fn from_string(s: &str) -> Result<URL, ()> {
        if !s.is_ascii() || s.is_empty() {
            return Err(());
        }

        let s = s.to_string();

        let (protocol, s) = s.split_once("://").unwrap_or(("", &s));
        let (user, s) = s.split_once("@").unwrap_or(("", s));
        let (host, resource) = s.split_once("/").ok_or(())?;
        let (host, port) = host.split_once(":").unwrap_or((host, ""));
        let (username, password) = user.split_once(":").unwrap_or(("", ""));

        let (resource, parameters) = resource.split_once("?").unwrap_or((resource, ""));

        let parameters: HashMap<String, String> = parameters.split("&")
            .filter(|s| !s.is_empty())
            .map(|p| p.split_once("=").unwrap_or((p, "")))
            .map(|(a, b)| (decode(a), decode(b)))
            .collect();

        Ok(Self {
            protocol: if protocol.is_empty() {
                None
            } else {
                Some(decode(protocol.to_string()))
            },
            username: if username.is_empty() {
                None
            } else {
                Some(decode(user.to_string()))
            },
            password: if password.is_empty() {
                None
            } else {
                Some(decode(password.to_string()))
            },
            host: if host.is_empty() {
                None
            } else {
                Some(decode(host.to_string()))
            },
            port: if port.is_empty() {
                None
            } else {
                Some(port.parse().map_err(|_| ())?)
            },
            resource: resource.split("/").filter(|s| !s.is_empty()).map(|s| decode(s)).collect(),
            parameters,
        })
    }

    pub fn protocol(&self) -> Option<&String> {
        self.protocol.as_ref()
    }

    pub fn with_protocol<S: Borrow<str>>(self, protocol: Option<S>) -> Self {
        Self {
            protocol: protocol.map(|s| s.borrow().to_string()),
            ..self
        }
    }

    pub fn username(&self) -> Option<&String> {
        self.username.as_ref()
    }

    pub fn with_username<S: Borrow<str>>(self, username: Option<S>) -> Self {
        Self {
            username: username.map(|s| s.borrow().to_string()),
            ..self
        }
    }

    pub fn password(&self) -> Option<&String> {
        self.password.as_ref()
    }

    pub fn with_password<S: Borrow<str>>(self, password: Option<S>) -> Self {
        Self {
            password: password.map(|s| s.borrow().to_string()),
            ..self
        }
    }

    pub fn host(&self) -> Option<&String> {
        self.host.as_ref()
    }

    pub fn with_host<S: Borrow<str>>(self, host: Option<S>) -> Self {
        Self {
            host: host.map(|s| s.borrow().to_string()),
            ..self
        }
    }

    pub fn port(&self) -> Option<&u16> {
        self.port.as_ref()
    }

    pub fn with_port(self, port: Option<u16>) -> Self {
        Self { port, ..self }
    }

    pub fn resource(&self) -> &Vec<String> {
        &self.resource
    }

    pub fn resource_string(&self) -> String {
        self.resource.clone().into_iter().reduce(|a, b| format!("{}/{}", a, b)).unwrap_or(String::from("/"))
    }

    pub fn with_resource<S: Borrow<str>>(self, resource: S) -> Self {
        let resource = resource.borrow();

        Self {
            resource: resource.split("/").filter(|s| !s.is_empty()).map(|s| s.to_string()).collect(),
            ..self
        }
    }

    pub fn push<S: Borrow<str>>(&mut self, s: S) {
        self.resource.push(s.borrow().to_owned());
    }

    pub fn pop(&mut self) {
        self.resource.pop();
    }

    pub fn param(&self, key: impl Borrow<str>) -> Option<&str> {
        self.parameters.get(key.borrow()).map(|s| s.as_str())
    }

    pub fn with_param(mut self, key: impl Borrow<str>, value: impl Borrow<str>) -> Self {
        self.parameters.insert(key.borrow().borrow().to_string(), value.borrow().to_string());
        self
    }

    pub fn as_string(&self) -> Result<String, ()> {
        let mut s = String::new();

        if let Some(p) = &self.protocol {
            s = format!("{}://", encode(p));
        }

        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            s += &format!("{}:{}", encode(user), encode(pass));
        }

        if let Some(host) = &self.host {
            s += &encode(host);
        }

        if let Some(port) = self.port {
            s += &format!(":{}", port);
        }

        s += &self.resource.iter().map(|s| encode(s)).fold(String::from(""), |a, b| format!("{}/{}", a, b));

        if !self.parameters.is_empty() {
            s += "?";
            s += &self.parameters.iter().map(|(key, value)| format!("{}={}", encode(key), encode(value))).reduce(|a, b| format!("{}&{}", a, b)).unwrap();
        }

        if s.is_ascii() {
            Ok(s)
        } else {
            Err(())
        }
    }
}

pub fn decode(s: impl Borrow<str>) -> String {
    let mut s = s.borrow().to_owned();

    while let Some(i) = s.find("%") {
        let (prefix, middle, suffix) = (
            (0..i)
                .map(|n| s.chars().nth(n).unwrap())
                .collect::<String>(),
            (i + 1..i + 3)
                .map(|n| s.chars().nth(n).unwrap())
                .collect::<String>(),
            (i + 3..s.len())
                .map(|n| s.chars().nth(n).unwrap())
                .collect::<String>(),
        );

        let decoded = u8::from_str_radix(&middle, 16).map_err(|_| ()).unwrap() as char;
        s = format!("{}{}{}", prefix, decoded, suffix);
    }

    return s;
}

pub fn encode<S: Borrow<str>>(s: &S) -> String {
    s.borrow()
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '~' | '.' => format! {"{}", c},
            _ => format!("%{:02X?}", (c as u8)),
        })
        .fold(String::new(), |a, b| format!("{}{}", a, b))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn url_encoding() {
        let url = URL::new()
            .with_protocol(Some("https"))
            .with_host(Some("justnoise.net"))
            .with_port(Some(25565))
            .with_resource("ch{ungus}/monkey picture.png");

        println!("{}", url.as_string().unwrap());
    }

    #[test]
    fn url_decoding() {
        let url = URL::from_string("https://justnoise.net:25565/el%20diablo/the%20devil.png?qualude=a_mile").unwrap();
        println!(
            "{} == {:?}",
            url.as_string().unwrap(),
            url
        );
    }
}
