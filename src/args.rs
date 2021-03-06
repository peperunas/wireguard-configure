use ipnet::IpNet;
use std::net::IpAddr;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(author)]
pub struct Arguments {
    #[structopt(subcommand)]
    pub subcommand: SubCommand,
    /// Configuration file to use
    #[structopt(
        name = "configuration-file",
        parse(from_os_str),
        short = "c",
        overrides_with = "configuration-name"
    )]
    pub config: Option<PathBuf>,
}

#[derive(StructOpt)]
pub enum SubCommand {
    /// Generate an example configuration file
    GenerateExample,
    /// List clients in this configuration
    List,
    /// Add a client to the configuration
    AddClient {
        /// Name of client to add
        client_name: String,
        /// Internal address for the new client
        #[structopt(short = "i")]
        internal_address: IpAddr,
        /// A list of subnets to be routed through the VPN for this client (e.g 10.0.0.1/32)
        #[structopt(required = true, short = "a")]
        allowed_ips: Vec<IpNet>,
        /// The DNS server to use
        #[structopt(short, long)]
        dns: Option<IpAddr>,
        /// Persistent keepalive for the client
        #[structopt(short, long)]
        persistent_keepalive: Option<usize>,
        /// Use the given public key, do not use an auto-generated key-pair
        #[structopt(long = "pub")]
        public_key: Option<String>,
    },
    /// Remove a client from the configuration
    RemoveClient {
        /// Name of client to remove
        #[structopt(required = true)]
        client_name: String,
    },
    /// Print the router configuration
    RouterConfig,
    /// Print the client configuration
    ClientConfig {
        /// Name of the client's configuration to print
        client_name: String,
    },
}
