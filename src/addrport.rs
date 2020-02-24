use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddrPort {
    pub address: String,
    pub port: u16,
}

impl AddrPort {
    pub fn new<A: Into<String>>(address: A, port: u16) -> AddrPort {
        AddrPort {
            address: address.into(),
            port: port,
        }
    }
}

impl fmt::Display for AddrPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}:{}", self.address, self.port)
    }
}
