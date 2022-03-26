use std::fmt::{Display, Formatter};

type DateTime = chrono::DateTime<chrono::Utc>;

pub struct Cookie {
    name: String,
    value: String,
    expiration: Option<DateTime>,
    http_only: bool,
    secure: bool,
}

impl Cookie {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            expiration: None,
            http_only: false,
            secure: false,
        }
    }

    pub fn with_expiration(self, expiration: DateTime) -> Self {
        Self { expiration: Some(expiration), .. self }
    }

    pub fn with_http_only(self, http_only: bool) -> Self {
        Self { http_only, .. self }
    }

    pub fn with_secure(self, secure: bool) -> Self {
        Self { secure, .. self }
    }
}

impl Display for Cookie {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.name, self.value)?;

        if let Some(expire) = &self.expiration {
            write!(f, "; Expire={}", expire.to_rfc2822())?;
        }

        if self.http_only {
            write!(f, "; HttpOnly")?;
        }

        if self.secure {
            write!(f, "; Secure")?;
        }

        Ok(())
    }
}
