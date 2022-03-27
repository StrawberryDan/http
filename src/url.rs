use std::borrow::Borrow;

#[derive(Debug, Clone)]
pub struct URL {
    protocol: Option<String>,
    username: Option<String>,
    password: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    resource: String,
}

impl URL {
    pub fn new() -> Self {
        Self {
            protocol: None,
            username: None,
            password: None,
            host: None,
            port: None,
            resource: "/".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Result<URL, ()> {
        if !s.is_ascii() || s.is_empty() {
            return Err(());
        }

        let mut s = s.to_string();
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

            let decoded = u8::from_str_radix(&middle, 16).map_err(|_| ())? as char;
            s = format!("{}{}{}", prefix, decoded, suffix);
        }

        let (protocol, s) = s.split_once("://").unwrap_or(("", &s));
        let (user, s) = s.split_once("@").unwrap_or(("", s));
        let (host, resource) = s.split_once("/").ok_or(())?;
        let (host, port) = host.split_once(":").unwrap_or((host, ""));
        let (username, password) = user.split_once(":").unwrap_or(("", ""));

        Ok(Self {
            protocol: if protocol.is_empty() {
                None
            } else {
                Some(protocol.to_string())
            },
            username: if username.is_empty() {
                None
            } else {
                Some(user.to_string())
            },
            password: if password.is_empty() {
                None
            } else {
                Some(password.to_string())
            },
            host: if host.is_empty() {
                None
            } else {
                Some(host.to_string())
            },
            port: if port.is_empty() {
                None
            } else {
                Some(port.parse().map_err(|_| ())?)
            },
            resource: format!("/{}", resource),
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

    pub fn resource(&self) -> &String {
        &self.resource
    }

    pub fn with_resource<S: Borrow<str>>(self, resource: S) -> Self {
        let resource = resource.borrow();
        if resource.starts_with("/") {
            Self {
                resource: resource.to_string(),
                ..self
            }
        } else {
            Self {
                resource: format!("/{}", resource),
                ..self
            }
        }
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

        s += &self.resource().split("/").filter(|s| !s.is_empty()).fold(String::new(), |a, b| format!("{}/{}", a, encode(&b)));

        if s.is_ascii() {
            Ok(s)
        } else {
            Err(())
        }
    }
}

fn encode<S: Borrow<str>>(s: &S) -> String {
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
        println!(
            "{:?}",
            URL::from_string("https://justnoise.net:25565/el%20diablo/the%20devil.png").unwrap()
        );
    }
}
