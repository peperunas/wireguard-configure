#![allow(dead_code)]

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
use ipnet::Ipv4Net;
use prettytable::{Cell, Row, Table};
use std::net::Ipv4Addr;
use std::path::Path;
use std::process::exit;
use structopt::StructOpt;

fn example_configuration() -> Configuration {
    // Router
    let router_ip: Ipv4Net = "10.0.1.1/24".parse().unwrap();
    let router_subnet: Ipv4Net = "10.0.1.0/24".parse().unwrap();

    // Client A
    let client_a_ip: Ipv4Addr = "10.0.1.2".parse().unwrap();
    let client_a_dns: Ipv4Addr = "10.0.1.1".parse().unwrap();
    let client_a_allowed_ips: Ipv4Net = "0.0.0.0/0".parse().unwrap();
    // Client B
    let client_b_ip: Ipv4Addr = "10.0.1.3".parse().unwrap();

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

fn main() {
    let args = Arguments::from_args();

    match args.subcommand {
        SubCommand::AddClient {
            configuration_path,
            client_name,
            internal_address,
            allowed_ips,
            dns,
            persistent_keepalive,
            public_key,
        } => {
            let mut config =
                Configuration::open(&configuration_path).expect("Failed to open configuration.");

            handle_add_client(
                &mut config,
                &configuration_path,
                &client_name,
                internal_address,
                allowed_ips,
                dns,
                persistent_keepalive,
                public_key,
            );
        }
        SubCommand::ClientConfig {
            configuration_path,
            client_name,
        } => {
            let config =
                Configuration::open(&configuration_path).expect("Failed to open configuration.");

            handle_client_config(&config, &client_name);
        }
        SubCommand::GenerateExample => {
            println!("{}", example_configuration());
        }
        SubCommand::List { configuration_path } => {
            let config =
                Configuration::open(&configuration_path).expect("Failed to open configuration.");

            handle_print(&config);
        }
        SubCommand::RemoveClient {
            configuration_path,
            client_name,
        } => {
            let mut config =
                Configuration::open(&configuration_path).expect("Failed to open configuration.");

            handle_remove_client(&mut config, &client_name, &configuration_path);
        }
        SubCommand::RouterConfig { configuration_path } => {
            let config =
                Configuration::open(&configuration_path).expect("Failed to open configuration.");

            handle_router_config(&config);
        }
    }
}

fn handle_add_client(
    config: &mut Configuration,
    out_config_path: &Path,
    client_name: &str,
    internal_address: Ipv4Addr,
    allowed_ips: Vec<Ipv4Net>,
    dns: Option<Ipv4Addr>,
    persistent_keepalive: Option<usize>,
    public_key: Option<String>,
) {
    // check if the client we are trying to add already exists
    if config
        .clients
        .iter()
        .any(|client| client.name == client_name)
    {
        eprintln!("Client {} already exists", client_name);
        return;
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

    config
        .save(out_config_path)
        .expect("Failed to save configuration.");

    println!("Client added");
}

fn handle_client_config(config: &Configuration, client_name: &str) {
    match config.client_by_name(client_name) {
        // TODO: change API
        Some(client) => println!(
            "{}",
            config.client_config(&client.name, &config.router).unwrap()
        ),
        None => println!("Could not find client {}", client_name),
    }
}

fn handle_print(config: &Configuration) {
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

fn handle_remove_client(config: &mut Configuration, client_name: &str, config_path: &Path) {
    if !config.remove_client_by_name(&client_name) {
        eprintln!("Failed to find and remove client {}", client_name);
        exit(1);
    }

    // TODO: properly handle errors
    config
        .save(config_path)
        .expect("Failed to save configuration.");

    println!("Client {} removed", client_name);
}

fn handle_router_config(config: &Configuration) {
    println!("{}\n", config.router.interface_str());

    for client in &config.clients {
        println!("{}\n", config.router.peer_str(&client));
    }
}
