use crate::addrport::AddrPort;
use ipnet::Ipv4Net;
use std::io::Write;
use std::net::Ipv4Addr;
use std::process::{Command, Stdio};

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
    pub private_key: String,
    pub public_key: String,
    pub external_address: AddrPort,
    pub internal_address: Ipv4Addr,
}

impl Router {
    pub fn new<S: Into<String>>(
        name: S,
        internal_address: Ipv4Addr,
        external_address: AddrPort,
    ) -> Router {
        let (private_key, public_key) = gen_keys().expect("Error while generating key pair.");

        Router {
            name: name.into(),
            private_key: private_key,
            public_key: public_key,
            external_address: external_address,
            internal_address: internal_address,
        }
    }

    pub fn set_external_address(&mut self, external_address: AddrPort) {
        self.external_address = external_address;
    }

    pub fn set_internal_address(&mut self, internal_address: Ipv4Addr) {
        self.internal_address = internal_address;
    }

    pub fn interface(&self) -> String {
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("# {}", self.name));
        lines.push("[Interface]".to_string());
        lines.push(format!("PrivateKey = {}", self.private_key));
        lines.push(format!("ListenPort = {}", self.external_address.port));
        lines.join("\n")
    }

    pub fn peer(&self, client: &Client) -> String {
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("# {}", client.name));
        lines.push("[Peer]".to_string());
        lines.push(format!("PublicKey = {}", client.public_key));

        if let Some(keepalive) = client.persistent_keepalive {
            lines.push(format!("PersistentKeepalive = {}", keepalive));
        }

        lines.push(format!("AllowedIPs = {}", client.internal_address));
        lines.join("\n")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Client {
    pub name: String,
    pub private_key: Option<String>,
    pub public_key: String,
    pub internal_address: Ipv4Addr,
    pub dns: Option<Ipv4Addr>,
    pub allowed_ips: Vec<Ipv4Net>,
    pub persistent_keepalive: Option<usize>,
}

impl Client {
    pub fn new<S: Into<String>>(name: S, internal_address: Ipv4Addr) -> Client {
        let (private_key, public_key) = gen_keys().expect("Error while generating key pair.");

        Client {
            name: name.into(),
            private_key: Some(private_key),
            public_key,
            internal_address,
            dns: None,
            allowed_ips: Vec::new(),
            persistent_keepalive: None,
        }
    }

    pub fn builder_push_allowed_ips(mut self, allowed_ip: Ipv4Net) -> Client {
        self.allowed_ips.push(allowed_ip);
        self
    }

    pub fn builder_persistent_keepalive(mut self, keepalive: Option<usize>) -> Client {
        self.persistent_keepalive = keepalive;
        self
    }

    pub fn builder_dns(mut self, dns: Option<Ipv4Addr>) -> Client {
        self.dns = dns;
        self
    }

    pub fn set_internal_address(&mut self, internal_address: Ipv4Addr) {
        self.internal_address = internal_address;
    }

    pub fn push_allowed_ip(&mut self, allowed_ip: Ipv4Net) {
        self.allowed_ips.push(allowed_ip);
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

    pub fn interface(&self) -> Option<String> {
        let mut lines: Vec<String> = Vec::new();

        match &self.private_key {
            Some(private_key) => {
                lines.push(format!("# {}", self.name));
                lines.push("[Interface]".to_string());
                lines.push(format!("PrivateKey = {}", private_key));
                lines.push(format!("Address = {}", self.internal_address));

                if let Some(dns) = self.dns {
                    lines.push(format!("DNS = {}", dns));
                }

                Some(lines.join("\n"))
            }
            None => None,
        }
    }

    pub fn peer(&self, router: &Router) -> String {
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("# {}", self.name));
        lines.push("[Peer]".to_string());
        lines.push(format!("PublicKey = {}", self.public_key));
        lines.push(format!(
            "Endpoint = {}:{}",
            router.external_address.address, router.external_address.port
        ));
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
