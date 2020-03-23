use ipnet::Ipv4Net;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(author)]
pub struct Arguments {
    #[structopt(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(StructOpt)]
pub enum SubCommand {
    /// Generate an example configuration file
    GenerateExample,
    /// List clients in this configuration
    List {
        /// wireguard-configure configuration file
        #[structopt(parse(from_os_str))]
        configuration_path: PathBuf,
    },
    /// Add a client to the configuration
    AddClient {
        /// wireguard-configure configuration file
        #[structopt(parse(from_os_str))]
        configuration_path: PathBuf,
        /// Name of client to add
        client_name: String,
        /// Internal address for the new client
        #[structopt(short = "i")]
        internal_address: Ipv4Addr,
        /// A list of subnets to be routed through the VPN for this client (e.g 10.0.0.1/32)
        #[structopt(required = true, short = "a")]
        allowed_ips: Vec<Ipv4Net>,
        /// The DNS server to use
        #[structopt(short, long)]
        dns: Option<Ipv4Addr>,
        /// Persistent keepalive for the client
        #[structopt(short, long)]
        persistent_keepalive: Option<usize>,
        /// Use the given public key, do not use an auto-generated key-pair
        #[structopt(long = "pub")]
        public_key: Option<String>,
    },
    /// Remove a client from the configuration
    RemoveClient {
        /// wireguard-configure configuration file
        #[structopt(parse(from_os_str))]
        configuration_path: PathBuf,
        /// Name of client to remove
        #[structopt(required = true)]
        client_name: String,
    },
    /// Print the router configuration
    RouterConfig {
        /// wireguard-configure configuration file
        #[structopt(parse(from_os_str))]
        configuration_path: PathBuf,
    },
    /// Print the client configuration
    ClientConfig {
        /// wireguard-configure configuration file
        #[structopt(parse(from_os_str))]
        configuration_path: PathBuf,
        /// Name of the client's configuration to print
        client_name: String,
    },
}
