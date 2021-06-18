use crate::addrport::AddrPort;
use ipnet::IpNet;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use std::fmt::Display;
use std::io::Write;
use std::net::IpAddr;
use std::process::{Command, Stdio};

#[derive(Clone, Debug, Serialize)]
pub enum TableType {
    Off,
    Auto,
    Custom(u32),
}

impl<'de> Deserialize<'de> for TableType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TableTypeVisitor;

        impl<'a> Visitor<'a> for TableTypeVisitor {
            type Value = TableType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "off, auto or a table number")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                match v.to_lowercase().as_str() {
                    "off" => Ok(TableType::Off),
                    "auto" => Ok(TableType::Auto),
                    x => {
                        let value = x.parse().map_err(serde::de::Error::custom)?;

                        Ok(TableType::Custom(value))
                    }
                }
            }
        }

        deserializer.deserialize_string(TableTypeVisitor)
    }
}

impl Display for TableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let out = match self {
            Self::Off => "off".to_owned(),
            Self::Auto => "auto".to_owned(),
            Self::Custom(value) => value.to_string(),
        };

        write!(f, "{}", out)
    }
}

fn gen_keys() -> Result<(String, String), std::io::Error> {
    let output = Command::new("wg").args(&["genkey"]).output()?;

    let privkey = String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .trim_start()
        .to_string();

    let mut command = Command::new("wg")
        .args(&["pubkey"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    command
        .stdin
        .as_mut()
        .expect("Failed to get stdin for wg pubkey")
        .write_all(privkey.as_bytes())?;

    let output = command.wait_with_output()?;

    let pubkey = String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .trim_start()
        .to_string();

    Ok((privkey, pubkey))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Router {
    pub name: String,
    pub internal_address: IpNet,
    pub external_address: AddrPort,
    pub private_key: String,
    pub public_key: String,
    pub mtu: Option<u16>,
    pub table: Option<TableType>,
    pub preup: Option<String>,
    pub postup: Option<String>,
    pub predown: Option<String>,
    pub postdown: Option<String>,
}

impl Router {
    pub fn new<S: Into<String>>(
        name: S,
        internal_address: IpNet,
        external_address: AddrPort,
    ) -> Router {
        // generating keypair by calling wg on the host system
        let (private_key, public_key) = gen_keys().expect("Error while generating key pair.");

        Router {
            name: name.into(),
            private_key,
            public_key,
            external_address,
            internal_address,
            mtu: None,
            table: None,
            preup: None,
            postup: None,
            predown: None,
            postdown: None,
        }
    }

    /*
     * Builder functions
     */

    pub fn with_mtu(mut self, mtu: Option<u16>) -> Router {
        self.mtu = mtu;
        self
    }

    pub fn with_table(mut self, table: Option<TableType>) -> Router {
        self.table = table;
        self
    }

    pub fn with_preup(mut self, preup: Option<String>) -> Router {
        self.preup = preup;
        self
    }

    pub fn with_postup(mut self, postup: Option<String>) -> Router {
        self.postup = postup;
        self
    }

    pub fn with_predown(mut self, predown: Option<String>) -> Router {
        self.predown = predown;
        self
    }

    pub fn with_postdown(mut self, postdown: Option<String>) -> Router {
        self.postdown = postdown;
        self
    }

    /*
     * Setters
     */

    pub fn set_external_address(&mut self, external_address: AddrPort) {
        self.external_address = external_address;
    }

    pub fn set_internal_address(&mut self, internal_address: IpNet) {
        self.internal_address = internal_address;
    }

    /*
     *
     */

    pub fn interface_str(&self) -> String {
        let mut lines: Vec<String> = Vec::new();

        // Router name
        lines.push(format!("# {}", self.name));

        // Interface section begins
        lines.push("[Interface]".to_string());

        // Internal address
        lines.push(format!("Address = {}", IpNet::from(self.internal_address)));

        // Private key
        lines.push(format!("PrivateKey = {}", self.private_key));

        // Listen port
        lines.push(format!("ListenPort = {}", self.external_address.port));

        // MTU, if any
        if let Some(mtu) = self.mtu {
            lines.push(format!("MTU = {}", mtu));
        }

        // Table, if any
        if let Some(table) = &self.table {
            lines.push(format!("Table = {}", table));
        }

        // PreUp, if any
        if let Some(preup) = &self.preup {
            lines.push(format!("PreUp = {}", preup));
        }

        // PostUp, if any
        if let Some(postup) = &self.postup {
            lines.push(format!("PostUp = {}", postup));
        }

        // PreDown, if any
        if let Some(predown) = &self.predown {
            lines.push(format!("PreDown = {}", predown));
        }

        // PostDown, if any
        if let Some(postdown) = &self.postdown {
            lines.push(format!("PostDown = {}", postdown));
        }

        lines.join("\n")
    }

    pub fn peer_str(&self, peer: &Peer) -> String {
        let mut lines: Vec<String> = Vec::new();

        // Peer name
        lines.push(format!("# {}", peer.name));

        // Peer section begins
        lines.push("[Peer]".to_string());

        // Public key
        lines.push(format!("PublicKey = {}", peer.public_key));

        // Allowed IPs
        lines.push(format!(
            "AllowedIPs = {}",
            IpNet::from(peer.internal_address)
        ));

        lines.join("\n")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Peer {
    pub name: String,
    pub internal_address: IpAddr,
    pub allowed_ips: Vec<IpNet>,
    pub dns: Option<IpAddr>,
    pub persistent_keepalive: Option<usize>,
    pub private_key: Option<String>,
    pub public_key: String,
    pub mtu: Option<u16>,
    pub table: Option<TableType>,
    pub preup: Option<String>,
    pub postup: Option<String>,
    pub predown: Option<String>,
    pub postdown: Option<String>,
}

impl Peer {
    pub fn new<S: Into<String>>(name: S, internal_address: IpAddr) -> Peer {
        // generating keypair by calling wg on the host system
        let (private_key, public_key) = gen_keys().expect("Error while generating key pair.");

        Peer {
            name: name.into(),
            private_key: Some(private_key),
            public_key,
            internal_address,
            dns: None,
            allowed_ips: Vec::new(),
            persistent_keepalive: None,
            mtu: None,
            table: None,
            preup: None,
            postup: None,
            predown: None,
            postdown: None,
        }
    }

    //
    // Builder functions
    //

    pub fn with_dns(mut self, dns: Option<IpAddr>) -> Peer {
        self.dns = dns;
        self
    }

    pub fn with_keepalive(mut self, keepalive: Option<usize>) -> Peer {
        self.persistent_keepalive = keepalive;
        self
    }

    pub fn with_vec_allowed_ips(mut self, allowed_ips: Vec<IpNet>) -> Peer {
        self.allowed_ips = allowed_ips;
        self
    }

    pub fn with_allowed_ips(mut self, allowed_ips: IpNet) -> Peer {
        self.allowed_ips.push(allowed_ips);
        self
    }

    pub fn with_mtu(mut self, mtu: Option<u16>) -> Peer {
        self.mtu = mtu;
        self
    }

    pub fn with_table(mut self, table: Option<TableType>) -> Peer {
        self.table = table;
        self
    }

    //
    // Setters
    //

    pub fn push_allowed_ip(&mut self, allowed_ips: IpNet) {
        self.allowed_ips.push(allowed_ips);
    }

    pub fn set_internal_address(&mut self, internal_address: IpAddr) {
        self.internal_address = internal_address;
    }

    pub fn set_persistent_keepalive(&mut self, keepalive: Option<usize>) {
        self.persistent_keepalive = keepalive;
    }

    pub fn set_private_key(&mut self, private_key: Option<String>) {
        self.private_key = private_key;
    }

    pub fn set_public_key(&mut self, public_key: String) {
        self.public_key = public_key;
    }

    //
    // Other functions
    //

    pub fn interface_str(&self) -> Option<String> {
        let mut lines: Vec<String> = Vec::new();

        match &self.private_key {
            Some(private_key) => {
                // Peer name
                lines.push(format!("# {}", self.name));

                // Interface section begins
                lines.push("[Interface]".to_string());

                // Private key
                lines.push(format!("PrivateKey = {}", private_key));

                // Internal address
                lines.push(format!("Address = {}", IpNet::from(self.internal_address)));

                // DNS, if any
                if let Some(dns) = self.dns {
                    lines.push(format!("DNS = {}", dns));
                }

                // MTU, if any
                if let Some(mtu) = self.mtu {
                    lines.push(format!("MTU = {}", mtu));
                }

                // Table, if any
                if let Some(table) = &self.table {
                    lines.push(format!("Table = {}", table));
                }
                // PreUp, if any
                if let Some(preup) = &self.preup {
                    lines.push(format!("PreUp = {}", preup));
                }

                // PostUp, if any
                if let Some(postup) = &self.postup {
                    lines.push(format!("PostUp = {}", postup));
                }

                // PreDown, if any
                if let Some(predown) = &self.predown {
                    lines.push(format!("PreDown = {}", predown));
                }

                // PostDown, if any
                if let Some(postdown) = &self.postdown {
                    lines.push(format!("PostDown = {}", postdown));
                }

                Some(lines.join("\n"))
            }
            // if no private key is present, we cannot produce a valid Interface section
            None => None,
        }
    }

    pub fn peer_str(&self, router: &Router) -> String {
        let mut lines: Vec<String> = Vec::new();

        // Router name
        lines.push(format!("# {}", router.name));

        // Peer section begins
        lines.push("[Peer]".to_string());

        // Public key
        lines.push(format!("PublicKey = {}", router.public_key));

        // Router endpoint
        lines.push(format!(
            "Endpoint = {}:{}",
            router.external_address.address, router.external_address.port
        ));

        // Keepalive, if any
        if let Some(keepalive) = self.persistent_keepalive {
            lines.push(format!("PersistentKeepalive = {}", keepalive));
        }

        // Allowed IPs
        lines.push(format!(
            "AllowedIPs = {}",
            self.allowed_ips
                .iter()
                .map(|ip| format!("{}", ip))
                .collect::<Vec<String>>()
                .join(", ")
        ));

        lines.join("\n")
    }
}
