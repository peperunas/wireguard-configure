#![allow(dead_code)]

#[macro_use]
extern crate serde_derive;

mod addrport;
mod args;
mod configuration;
mod endpoint;

use crate::addrport::AddrPort;
use crate::configuration::Configuration;
use crate::endpoint::{EndPoint, Router};
use args::{Arguments, SubCommand};
use prettytable::{cell::Cell, row::Row, Table};
use std::path::Path;
use std::process::exit;
use structopt::StructOpt;

fn example_configuration() -> Configuration {
    let router = Router::new(
        "vpn-router",
        "10.0.0.1".parse().unwrap(),
        AddrPort::new("vpn.com", 47654),
    );

    let mut configuration = Configuration::new(router);

    configuration.push_client(
        EndPoint::new("client-a", "10.0.1.1".parse().unwrap())
            .builder_push_allowed_ips("10.0.1.0/24".parse().unwrap())
            .builder_persistent_keepalive(Some(25))
            .builder_dns(Some("10.0.0.1".parse().unwrap())),
    );

    configuration.push_client(
        EndPoint::new("client-b", "10.0.2.1".parse().unwrap())
            .builder_push_allowed_ips("10.0.2.0/24".parse().unwrap())
            .builder_persistent_keepalive(Some(25)),
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
            dns: _,
            persistent_keepalive,
            public_key,
            private_key: _,
        } => {
            let mut configuration = Configuration::open(&args.config);
            if configuration
                .clients()
                .iter()
                .any(|client| client.name() == client_name)
            {
                eprintln!("Client {} already exists", client_name);
                exit(1);
            }

            let mut endpoint = EndPoint::new(client_name, internal_address);

            if let Some(public_key) = public_key {
                endpoint.set_private_key(None);
                endpoint.set_public_key(public_key.to_string());
            }

            if let Some(keepalive) = persistent_keepalive {
                endpoint.set_persistent_keepalive(Some(keepalive));
            }

            for allowed_ip in allowed_ips {
                endpoint.push_allowed_ip(allowed_ip);
            }

            // TODO: Add DNS and private key handling

            configuration.push_client(endpoint);

            configuration.save(&args.config);

            println!("Client added");
        }
        SubCommand::ClientConfig { client_name } => {
            let configuration = Configuration::open(&args.config);

            configuration
                .client_by_name(&client_name)
                .expect(&format!("Could not find client {}", client_name));

            println!("{}", configuration.client_config(&client_name).unwrap());
        }
        SubCommand::GenerateExample => {
            example_configuration().save(Path::new(&args.config));
            println!("Configuration saved to file.");
        }
        SubCommand::List => {
            let configuration = Configuration::open(&args.config);

            let mut table = Table::new();

            table.add_row(Row::new(vec![
                Cell::new("Name"),
                Cell::new("Internal Address"),
                Cell::new("Allowed IPs"),
            ]));

            table.add_row(Row::new(vec![
                Cell::new(configuration.router().name()),
                Cell::new(&format!("{}", configuration.router().internal_address())),
                Cell::new(""),
            ]));

            for client in configuration.clients() {
                table.add_row(Row::new(vec![
                    Cell::new(client.name()),
                    Cell::new(&format!("{}", client.internal_address())),
                    Cell::new(
                        &client
                            .allowed_ips()
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
            let mut configuration = Configuration::open(&args.config);

            if !configuration.remove_client_by_name(&client_name) {
                eprintln!("Failed to find and remove client {}", client_name);
                exit(1);
            }

            configuration.save(&args.config);
            println!("Client {} removed", client_name);
        }
        SubCommand::RouterConfig => {
            let configuration = Configuration::open(&args.config);

            println!("{}", configuration.router().interface());

            for client in configuration.clients() {
                println!("{}", client.peer());
            }
        }
    }
}
