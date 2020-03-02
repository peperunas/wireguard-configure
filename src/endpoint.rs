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
    pub internal_address: Ipv4Net,
}

impl Router {
    pub fn new<S: Into<String>>(
        name: S,
        internal_address: Ipv4Net,
        external_address: AddrPort,
    ) -> Router {
        // generating keypair by calling wg on the host system
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

    pub fn set_internal_address(&mut self, internal_address: Ipv4Net) {
        self.internal_address = internal_address;
    }

    pub fn interface(&self) -> String {
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("# {}", self.name));
        lines.push("[Interface]".to_string());
        lines.push(format!(
            "Address = {}",
            Ipv4Net::from(self.internal_address)
        ));
        lines.push(format!("PrivateKey = {}", self.private_key));
        lines.push(format!("ListenPort = {}", self.external_address.port));
        lines.join("\n")
    }

    pub fn peer(&self, client: &Peer) -> String {
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("# {}", client.name));
        lines.push("[Peer]".to_string());
        lines.push(format!("PublicKey = {}", client.public_key));

        if let Some(keepalive) = client.persistent_keepalive {
            lines.push(format!("PersistentKeepalive = {}", keepalive));
        }

        lines.push(format!(
            "AllowedIPs = {}",
            Ipv4Net::from(client.internal_address)
        ));
        lines.join("\n")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Peer {
    pub allowed_ips: Vec<Ipv4Net>,
    pub dns: Option<Ipv4Addr>,
    pub internal_address: Ipv4Addr,
    pub name: String,
    pub persistent_keepalive: Option<usize>,
    pub private_key: Option<String>,
    pub public_key: String,
}

impl Peer {
    pub fn new<S: Into<String>>(name: S, internal_address: Ipv4Addr) -> Peer {
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
        }
    }

    //
    // Builder functions
    //

    pub fn builder_dns(mut self, dns: Option<Ipv4Addr>) -> Peer {
        self.dns = dns;
        self
    }

    pub fn builder_keepalive(mut self, keepalive: Option<usize>) -> Peer {
        self.persistent_keepalive = keepalive;
        self
    }

    pub fn builder_vec_allowed_ips(mut self, allowed_ips: Vec<Ipv4Net>) -> Peer {
        self.allowed_ips = allowed_ips;
        self
    }

    pub fn builder_allowed_ips(mut self, allowed_ip: Ipv4Net) -> Peer {
        self.allowed_ips.push(allowed_ip);
        self
    }

    //
    // Setters
    //

    pub fn push_allowed_ip(&mut self, allowed_ip: Ipv4Net) {
        self.allowed_ips.push(allowed_ip);
    }

    pub fn set_internal_address(&mut self, internal_address: Ipv4Addr) {
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
                lines.push(format!("Address = {}", self.internal_address));

                // DNS, if any
                if let Some(dns) = self.dns {
                    lines.push(format!("DNS = {}", dns));
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
        lines.push(format!("PublicKey = {}", self.public_key));

        // Router endpoint
        lines.push(format!(
            "Endpoint = {}:{}",
            router.external_address.address, router.external_address.port
        ));

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
