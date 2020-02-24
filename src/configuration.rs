use crate::endpoint::{Client, Router};
use ipnet::Ipv4Net;
use serde_yaml;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    pub router: Router,
    pub clients: Vec<Client>,
}

impl Configuration {
    pub fn open(path: &Path) -> Result<Configuration, Box<dyn Error>> {
        let mut file = File::open(path).expect(&format!("Failed to open {:?}", path));
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
            router: router,
            clients: Vec::new(),
        }
    }

    pub fn push_client(&mut self, client: Client) {
        self.clients.push(client);
    }

    pub fn remove_client_by_name(&mut self, name: &str) -> bool {
        for i in 0..self.clients.len() {
            if self.clients[i].name == name {
                self.clients.remove(i);
                return true;
            }
        }
        false
    }

    pub fn client_by_name(&self, name: &str) -> Option<&Client> {
        self.clients.iter().find(|client| client.name == name)
    }

    pub fn all_allowed_ips(&self) -> Vec<Ipv4Net> {
        self.clients
            .iter()
            .flat_map(|client| client.allowed_ips.clone())
            .collect()
    }

    pub fn client_config(&self, name: &str, router: &Router) -> Option<String> {
        let client = self.client_by_name(name)?;

        match client.interface() {
            Some(interface) => Some(format!("{}\n\n{}", interface, client.peer(&router))),
            None => None,
        }
    }
}
