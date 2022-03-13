use crate::Error;

#[derive(Debug)]
pub struct URL {
    protocol: String,
    username: String,
    password: String,
    host: String,
    resource: String,
}

impl URL {
    pub fn from_string(s: &str) -> Result<URL, Error> {
        let (protocol, s) = s.split_once("://").unwrap_or(("", s));
        let (user, s) = s.split_once("@").unwrap_or(("", s));
        let (host, resource) = s.split_once("/").ok_or(Error::URLParse)?;
        let (username, password) = user.split_once(":").unwrap_or(("", ""));
        Ok( Self {
            protocol: protocol.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            host: host.to_string(),
            resource: format!("/{}", resource)
        } )
    }

    pub fn protocol(&self) -> &String {
        &self.protocol
    }

    pub fn username(&self) -> &String {
        &self.username
    }

    pub fn password(&self) -> &String {
        &self.password
    }

    pub fn host(&self) -> &String {
        &self.host
    }

    pub fn resource(&self) -> &String {
        &self.resource
    }
}

impl URL {

}

