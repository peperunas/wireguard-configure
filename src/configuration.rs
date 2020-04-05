use crate::endpoint::{Peer, Router};
use serde_yaml;
use std::error::Error;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

pub const WIREGUARD_CONFIG_PATH: &str = "/etc/wireguard";

#[derive(Clone, Debug, Deserialize, Serialize, StructOpt)]
pub struct ConfigOpts {
    /// A wireguard-configure configuration file name found in /etc/wireguard. The file must end in .toml.
    /// A configuration is named after its file stem.
    ///
    /// e.g: wg0 -> /etc/wireguard/wg0.toml
    #[structopt(name = "configuration-name", required_unless = "custom-config-path")]
    pub name: Option<String>,

    /// Use a manually specified configuration file
    #[structopt(
        name = "custom-config-path",
        parse(from_os_str),
        short = "c",
        overrides_with = "configuration-name"
    )]
    pub path: Option<PathBuf>,
}

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
        println!("Opening {:?}", path);

        let mut file = File::open(path)?;
        let mut buffer: String = String::new();

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

    pub fn from_name(name: &str) -> Result<Configuration, Box<dyn Error>> {
        let mut config_path = PathBuf::from(WIREGUARD_CONFIG_PATH);
        // appending file stem and extension to configuration folder path
        config_path.push(name);
        config_path.set_extension("toml");

        println!("Opening {:?}", config_path);

        // checking if file exists
        if !config_path.is_file() {
            return Err(format!("{} is not a file", config_path.to_str().unwrap()))?;
        }

        Configuration::from_path(&config_path)
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        // extracting path from metadata
        let path = match &self.metadata {
            None => return Err("Configuration metadata not found.")?,
            Some(metadata) => match &metadata.path {
                None => return Err("No path defined for this configuration.")?,
                Some(path) => path,
            },
        };

        let mut file = OpenOptions::new().read(true).write(true).open(path)?;
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
        match &mut self.metadata {
            Some(metadata) => metadata.name = Some(name.to_string()),
            None => {
                self.metadata = Some(ConfigOpts {
                    name: Some(name.to_string()),
                    path: None,
                })
            }
        }

        self
    }

    pub fn with_path(mut self, path: &Path) -> Configuration {
        match &mut self.metadata {
            Some(metadata) => metadata.path = Some(path.to_path_buf()),
            None => {
                self.metadata = Some(ConfigOpts {
                    name: None,
                    path: Some(path.to_path_buf()),
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
