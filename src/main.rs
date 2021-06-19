#[macro_use]
extern crate serde_derive;

mod addrport;
mod args;
mod configuration;
mod endpoint;

use crate::addrport::AddrPort;
use crate::configuration::Configuration;
use crate::endpoint::{Peer, Router};
use args::{Arguments, SubCommand};
use atty::Stream;
use ipnet::IpNet;
use prettytable::{Cell, Row, Table};
use std::error::Error;
use std::io::Read;
use std::net::IpAddr;
use structopt::StructOpt;

fn example_configuration() -> Configuration {
    // Router
    let router_ip = "10.0.1.1/24".parse().unwrap();
    let router_subnet = "10.0.1.0/24".parse().unwrap();

    // Client A
    let client_a_ip = "10.0.1.2".parse().unwrap();
    let client_a_dns = "10.0.1.1".parse().unwrap();
    let client_a_allowed_ips = "0.0.0.0/0".parse().unwrap();

    // Client B
    let client_b_ip = "10.0.1.3".parse().unwrap();

    let router = Router::new("vpn-router", router_ip, AddrPort::new("vpn.com", 31337));
    let mut configuration = Configuration::new(router);

    configuration.push_peer(
        Peer::new("client-a", client_a_ip)
            .with_allowed_ips(client_a_allowed_ips)
            .with_keepalive(Some(25))
            .with_dns(Some(client_a_dns)),
    );

    configuration.push_peer(
        Peer::new("client-b", client_b_ip)
            .with_allowed_ips(router_subnet)
            .with_keepalive(Some(25)),
    );

    configuration
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::from_args();

    if let SubCommand::GenerateExample = args.subcommand {
        println!("{}", example_configuration());
        return Ok(());
    }

    // retrieve configuration either from config file (if specified) or stdin
    let mut config = match args.config {
        // from config file
        Some(config) => Configuration::from_path(&config)?,
        // from stdin
        None => {
            // check if we are a tty
            if is_tty() {
                println!("No configuration opened. Select a configuration file or pipe a configuration to stdin.");
                return Ok(());
            }

            let stdin = std::io::stdin();
            let mut stdin_data = String::new();

            stdin.lock().read_to_string(&mut stdin_data)?;

            serde_yaml::from_str(&stdin_data)?
        }
    };

    match args.subcommand {
        SubCommand::AddClient {
            client_name,
            internal_address,
            allowed_ips,
            dns,
            persistent_keepalive,
            public_key,
        } => {
            handle_add_client(
                &mut config,
                &client_name,
                internal_address,
                allowed_ips,
                dns,
                persistent_keepalive,
                public_key,
            )
            .expect("Failed to add client.");

            Ok(())
        }
        SubCommand::ClientConfig { client_name } => {
            handle_client_config(&config, &client_name);
            Ok(())
        }
        // TODO: ugly
        SubCommand::GenerateExample => {
            println!("{}", example_configuration());
            Ok(())
        }
        SubCommand::List => {
            handle_list(&config);
            Ok(())
        }
        SubCommand::RemoveClient { client_name } => {
            handle_remove_client(&mut config, &client_name).expect("Failed to remove client.");
            Ok(())
        }
        SubCommand::RouterConfig => {
            handle_router_config(&config);
            Ok(())
        }
    }
}

fn handle_add_client(
    config: &mut Configuration,
    client_name: &str,
    internal_address: IpAddr,
    allowed_ips: Vec<IpNet>,
    dns: Option<IpAddr>,
    persistent_keepalive: Option<usize>,
    public_key: Option<String>,
) -> Result<(), Box<dyn Error>> {
    // check if the client we are trying to add already exists
    if config
        .clients
        .iter()
        .any(|client| client.name == client_name)
    {
        eprintln!("Client {} already exists", client_name);
        return Ok(());
    }

    // creating peer
    let mut peer = Peer::new(client_name, internal_address)
        .with_dns(dns)
        .with_keepalive(persistent_keepalive)
        .with_vec_allowed_ips(allowed_ips);

    if let Some(public_key) = public_key {
        peer.set_private_key(None);
        peer.set_public_key(public_key);
    }

    // updating configuration
    config.push_peer(peer);

    config.save()?;

    if !config.is_from_tty() {
        println!("Client added");
    }

    Ok(())
}

fn handle_client_config(config: &Configuration, client_name: &str) {
    match config.client_config(client_name) {
        Some(config) => println!("{}", config),
        None => println!("Could not find client {}", client_name),
    }
}

fn handle_list(config: &Configuration) {
    let mut table = Table::new();

    table.add_row(Row::new(vec![
        Cell::new("Name"),
        Cell::new("Internal Address"),
        Cell::new("Allowed IPs"),
    ]));

    table.add_row(Row::new(vec![
        Cell::new(&config.router.name),
        Cell::new(&format!("{}", config.router.internal_address)),
        Cell::new(""),
    ]));

    for client in &config.clients {
        table.add_row(Row::new(vec![
            Cell::new(&client.name),
            Cell::new(&format!("{}", client.internal_address)),
            Cell::new(
                &client
                    .allowed_ips
                    .iter()
                    .map(|ip| format!("{}", ip))
                    .collect::<Vec<String>>()
                    .join(","),
            ),
        ]));
    }

    table.printstd();
}

fn handle_remove_client(
    config: &mut Configuration,
    client_name: &str,
) -> Result<(), Box<dyn Error>> {
    let old_clients_len = config.clients.len();

    config.clients.retain(|x| x.name != client_name);

    if config.clients.len() == old_clients_len {
        println!("Could not find and remove client \"{}\"", client_name);
        return Ok(());
    }

    config.save()?;

    if !config.is_from_tty() {
        println!("Client {} removed", client_name);
    }

    Ok(())
}

fn handle_router_config(config: &Configuration) {
    println!("{}\n", config.router.interface_str());

    for client in &config.clients {
        println!("{}\n", config.router.peer_str(&client));
    }
}

fn is_tty() -> bool {
    atty::is(Stream::Stdin)
}
