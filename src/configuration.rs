use crate::endpoint::{Peer, Router};
use ipnet::Ipv4Net;
use serde_yaml;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    pub router: Router,
    pub clients: Vec<Peer>,
}

impl fmt::Display for Configuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_yaml::to_string(&self).expect("Failed to serialize configuration.")
        )
    }
}

impl Configuration {
    pub fn open(path: &Path) -> Result<Configuration, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut buffer: String = String::new();

        file.read_to_string(&mut buffer)?;

        Ok(serde_yaml::from_str(&buffer)?)
    }

    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let mut file = File::create(path)?;

        let bytes = serde_yaml::to_string(&self).expect("Failed to serialize configuration");

        file.write_all(bytes.as_bytes())?;

        Ok(())
    }

    pub fn new(router: Router) -> Configuration {
        Configuration {
            router,
            clients: Vec::new(),
        }
    }

    pub fn push_peer(&mut self, client: Peer) {
        self.clients.push(client);
    }

    pub fn client_by_name(&self, name: &str) -> Option<&Peer> {
        self.clients.iter().find(|client| client.name == name)
    }

    pub fn client_config(&self, name: &str) -> Option<String> {
        let client = self.client_by_name(name)?;

        client
            .interface_str()
            .map(|interface| format!("{}\n\n{}", interface, client.peer_str(&self.router)))
    }
}
