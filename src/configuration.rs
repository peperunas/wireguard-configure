use crate::endpoint::{Peer, Router};
use ipnet::Ipv4Net;
use serde_yaml;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::args::ConfigOpts;

pub const WIREGUARD_CONFIG_PATH: &Path = &Path::new("/etc/wireguard");

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Configuration {
    // Do not serialize metadata
    #[serde(skip_serializing)]
    pub metadata: Option<ConfigOpts>,
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
    pub fn from_path(path: &Path) -> Result<Configuration, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut buffer: String = String::new();

        if !path.is_file() {
            return Err("The provided path is not a file.")?;
        }

        let extension = path
            .extension()
            .expect("The provided path has an invalid extension.");

        if extension != "toml" {
            return Err("The provided path does not end in .toml")?;
        }

        // extracting the configuration name from the file stem, if valid
        let config_name = path
            .file_stem()
            .expect("Invalid file stem.")
            .to_str()
            .expect("Cannot parse file stem.");

        // reading file contents
        file.read_to_string(&mut buffer)?;

        // deserializing file contents
        let buf_config: Configuration = serde_yaml::from_str(&buffer)?;

        // adding metadata to config
        let config = buf_config.with_name(config_name).with_path(path);

        Ok(config)
    }

    pub fn from_name(name: String) -> Result<Configuration, Box<dyn Error>> {
        // Building configuration file path from name.
        // Checking if the configuration file exists
        // in the configuration folder, and if so,
        // parsing its contents.
        let config_path = WIREGUARD_CONFIG_PATH.to_path_buf();

        // appending file stem and extension
        config_path.push(format!("{}.toml", name));

        // checking if file exists
        if !config_path.is_file() {
            return Err(format!(
                "The configuration file {} does not exist.",
                config_path.to_str().unwrap()
            ))?;
        }

        Configuration::from_path(&config_path)
    }

    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let mut file = File::create(path)?;

        let bytes = serde_yaml::to_string(&self).expect("Failed to serialize configuration");

        file.write_all(bytes.as_bytes())?;

        Ok(())
    }

    pub fn new(router: Router) -> Configuration {
        Configuration {
            metadata: None,
            router,
            clients: Vec::new(),
        }
    }

    pub fn with_name<S>(mut self, name: S) -> Configuration
    where
        S: ToString,
    {
        match self.metadata {
            Some(metadata) => metadata.name = Some(name.to_string()),
            None => {
                self.metadata = Some(ConfigOpts {
                    name: Some(name.to_string()),
                    custom_config_path: None,
                })
            }
        }

        self
    }

    pub fn with_path(mut self, path: &Path) -> Configuration {
        match self.metadata {
            Some(metadata) => metadata.custom_config_path = Some(path.to_path_buf()),
            None => {
                self.metadata = Some(ConfigOpts {
                    name: None,
                    custom_config_path: Some(path.to_path_buf()),
                })
            }
        }

        self
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
