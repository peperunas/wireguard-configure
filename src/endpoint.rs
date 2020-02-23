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
    name: String,
    private_key: String,
    public_key: String,
    external_address: AddrPort,
    internal_address: Ipv4Addr,
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn private_key(&self) -> &str {
        &self.private_key
    }

    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    pub fn external_address(&self) -> &AddrPort {
        &self.external_address
    }

    pub fn internal_address(&self) -> &Ipv4Addr {
        &self.internal_address
    }

    pub fn interface(&self) -> String {
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("# {}", self.name()));
        lines.push("[Interface]".to_string());
        lines.push(format!("PrivateKey = {}", self.private_key()));
        lines.push(format!("ListenPort = {}", self.external_address().port()));
        lines.join("\n")
    }

    pub fn peer(&self, of: &EndPoint, allowed_ips: &[Ipv4Net]) -> String {
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("# {}", self.name()));
        lines.push("[Peer]".to_string());
        lines.push(format!("PublicKey = {}", self.public_key()));
        lines.push(format!("Endpoint = {}", self.external_address()));

        if let Some(keepalive) = of.persistent_keepalive() {
            lines.push(format!("PersistentKeepalive = {}", keepalive));
        }

        lines.push(format!(
            "AllowedIPs = {}",
            allowed_ips
                .into_iter()
                .map(|ip| format!("{}", ip))
                .collect::<Vec<String>>()
                .join(", ")
        ));
        lines.join("\n")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EndPoint {
    name: String,
    private_key: Option<String>,
    public_key: String,
    external_address: Option<AddrPort>,
    internal_address: Ipv4Addr,
    dns: Option<Ipv4Addr>,
    allowed_ips: Vec<Ipv4Net>,
    persistent_keepalive: Option<usize>,
}

impl EndPoint {
    pub fn new<S: Into<String>>(name: S, internal_address: Ipv4Addr) -> EndPoint {
        let (private_key, public_key) = gen_keys().expect("Error while generating key pair.");
        EndPoint {
            name: name.into(),
            private_key: Some(private_key),
            public_key: public_key,
            external_address: None,
            internal_address: internal_address,
            dns: None,
            allowed_ips: Vec::new(),
            persistent_keepalive: None,
        }
    }

    pub fn builder_external_address(mut self, external_address: Option<AddrPort>) -> EndPoint {
        self.external_address = external_address;
        self
    }

    pub fn builder_push_allowed_ips(mut self, allowed_ip: Ipv4Net) -> EndPoint {
        self.allowed_ips.push(allowed_ip);
        self
    }

    pub fn builder_persistent_keepalive(mut self, keepalive: Option<usize>) -> EndPoint {
        self.persistent_keepalive = keepalive;
        self
    }

    pub fn builder_dns(mut self, dns: Option<Ipv4Addr>) -> EndPoint {
        self.dns = dns;
        self
    }

    pub fn set_external_address(&mut self, external_address: Option<AddrPort>) {
        self.external_address = external_address;
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

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn private_key(&self) -> Option<&str> {
        self.private_key.as_ref().map(|s| s.as_str())
    }
    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    pub fn external_address(&self) -> Option<&AddrPort> {
        self.external_address.as_ref()
    }

    pub fn internal_address(&self) -> &Ipv4Addr {
        &self.internal_address
    }

    pub fn allowed_ips(&self) -> Vec<Ipv4Net> {
        if !self.allowed_ips.is_empty() {
            self.allowed_ips.clone()
        } else {
            vec![Ipv4Net::new(self.internal_address().clone(), 32)
                .expect("Failed to make Ipv4Net for allowed_ips()")]
        }
    }

    pub fn persistent_keepalive(&self) -> Option<usize> {
        self.persistent_keepalive.clone()
    }

    pub fn interface(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("# {}", self.name()));
        lines.push("[Interface]".to_string());
        lines.push(format!(
            "PrivateKey = {}",
            self.private_key().unwrap_or("USER_SUPPLIED")
        ));
        lines.push(format!("Address = {}", self.internal_address));

        if let Some(dns) = self.dns {
            lines.push(format!("DNS = {}", dns));
        }

        if let Some(external_address) = self.external_address() {
            lines.push(format!("ListenPort = {}", external_address.port()));
        }

        lines.join("\n")
    }

    pub fn peer(&self) -> String {
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("# {}", self.name()));
        lines.push("[Peer]".to_string());
        lines.push(format!("PublicKey = {}", self.public_key()));
        if let Some(external_address) = self.external_address() {
            lines.push(format!("Endpoint = {}", external_address));
        }

        lines.push(format!(
            "AllowedIPs = {}",
            self.allowed_ips()
                .iter()
                .map(|ip| format!("{}", ip))
                .collect::<Vec<String>>()
                .join(", ")
        ));
        lines.join("\n")
    }
}
