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
            client_name,
            internal_address,
            allowed_ips,
            dns,
            persistent_keepalive,
            public_key,
        } => {
            let mut configuration =
                Configuration::open(&args.config).expect("Failed to open configuration.");

            // check if the client we are trying to add already exists
            if configuration
                .clients
                .iter()
                .any(|client| client.name == client_name)
            {
                eprintln!("Client {} already exists", client_name);
                exit(1);
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
            configuration.push_peer(peer);

            configuration
                .save(&args.config)
                .expect("Failed to save configuration.");

            println!("Client added");
        }
        SubCommand::ClientConfig { client_name } => {
            // TODO: fixme
            let configuration =
                Configuration::open(&args.config).expect("Failed to open configuration.");

            configuration
                .client_by_name(&client_name)
                .expect(&format!("Could not find client {}", client_name));

            println!(
                "{}",
                configuration
                    .client_config(&client_name, &configuration.router)
                    .unwrap()
            );
        }
        SubCommand::GenerateExample => {
            // TODO: properly handle errors
            example_configuration()
                .save(Path::new(&args.config))
                .expect("Failed to save configuration.");
            println!("Configuration saved to file.");
        }
        SubCommand::List => {
            // TODO: fixme
            let configuration =
                Configuration::open(&args.config).expect("Failed to open configuration.");

            let mut table = Table::new();

            table.add_row(Row::new(vec![
                Cell::new("Name"),
                Cell::new("Internal Address"),
                Cell::new("Allowed IPs"),
            ]));

            table.add_row(Row::new(vec![
                Cell::new(&configuration.router.name),
                Cell::new(&format!("{}", configuration.router.internal_address)),
                Cell::new(""),
            ]));

            for client in configuration.clients {
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
        SubCommand::RemoveClient { client_name } => {
            // TODO: fixme
            let mut configuration =
                Configuration::open(&args.config).expect("Failed to open configuration.");

            if !configuration.remove_client_by_name(&client_name) {
                eprintln!("Failed to find and remove client {}", client_name);
                exit(1);
            }

            // TODO: properly handle errors
            configuration
                .save(&args.config)
                .expect("Failed to save configuration.");
            println!("Client {} removed", client_name);
        }
        SubCommand::RouterConfig => {
            // TODO: fixme
            let configuration =
                Configuration::open(&args.config).expect("Failed to open configuration.");

            println!("{}\n", configuration.router.interface_str());

            for client in configuration.clients {
                println!("{}\n", configuration.router.peer_str(&client));
            }
        }
    }
}
