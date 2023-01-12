use super::super::{VitInteractiveCommandExec, VitUserInteractionController};
use crate::Result;
use clap::Parser;
use hersir::controller::interactive::args::describe::{
    DescribeNodes, DescribeTopology, DescribeVotePlans, DescribeWallets,
};

#[derive(Parser, Debug)]
pub enum Describe {
    /// Prints available wallets with aliases
    /// that can be used
    Wallets(DescribeWallets),
    /// Prints available node with aliases
    /// that can be used
    Nodes(DescribeNodes),
    /// Prints trusted peer info
    Topology(DescribeTopology),
    /// Prints everything
    All(DescribeAll),
    /// Prints Vit Stations
    Stations(DescribeVitStations),
    /// Prints wallet proxies
    Proxies(DescribeWalletProxies),
    /// Prints Votes Plan
    VotePlan(DescribeVotePlans),
}

impl Describe {
    pub fn exec(&self, command: &mut VitInteractiveCommandExec) -> Result<()> {
        match self {
            Describe::Wallets(wallets) => {
                wallets.exec(command.controller_mut())?;
            }
            Describe::Nodes(desc_nodes) => {
                desc_nodes.exec(command.controller_mut())?;
            }
            Describe::All(all) => {
                all.exec(command)?;
            }
            Describe::Topology(topology) => {
                topology.exec(command.controller_mut())?;
            }
            Describe::VotePlan(vote_plans) => {
                vote_plans.exec(command.controller_mut())?;
            }
            Describe::Stations(stations) => stations.exec(command.vit_controller_mut()),
            Describe::Proxies(proxies) => proxies.exec(command.vit_controller_mut()),
        };
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct DescribeVitStations {
    #[clap(short = 'a', long = "alias")]
    pub alias: Option<String>,
}

impl DescribeVitStations {
    pub fn exec(&self, controller: &mut VitUserInteractionController) {
        println!("Vit Stations:");
        for vit_station in controller.vit_stations() {
            println!(
                "\t{}: rest api: {}",
                vit_station.alias(),
                vit_station.address()
            );
        }
    }
}

#[derive(Parser, Debug)]
pub struct DescribeAll {
    #[clap(short = 'a', long = "alias")]
    pub alias: Option<String>,
}

impl DescribeAll {
    pub fn exec(&self, command: &mut VitInteractiveCommandExec) -> Result<()> {
        let describe_wallets = DescribeWallets { alias: None };
        describe_wallets.exec(command.controller_mut())?;
        let describe_nodes = DescribeNodes { alias: None };
        describe_nodes.exec(command.controller_mut())?;
        let describe_wallet_proxies = DescribeWalletProxies { alias: None };
        describe_wallet_proxies.exec(command.vit_controller_mut());
        let describe_vit_stations = DescribeVitStations { alias: None };
        describe_vit_stations.exec(command.vit_controller_mut());
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct DescribeWalletProxies {
    #[clap(short = 'a', long = "alias")]
    pub alias: Option<String>,
}

impl DescribeWalletProxies {
    pub fn exec(&self, controller: &mut VitUserInteractionController) {
        println!("Proxies:");
        for proxy in controller.proxies() {
            println!(
                "\t{}: rest api: {}",
                proxy.alias(),
                proxy.settings().proxy_address
            );
        }
    }
}
