use crate::{
    args::Args,
    config::Config,
    controller::{MonitorControllerBuilder, ProgressBarController},
    error::Error,
};
use jormungandr_automation::jormungandr::JormungandrRest;
use slave_pool::ThreadPool;
use std::sync::mpsc::channel;

pub fn spawn_network(mut config: Config, args: Args) -> Result<(), Error> {
    let mut topology = config.build_topology();
    let (tx, rx) = channel();

    let mut monitor_controller = MonitorControllerBuilder::new(&config.session.title)
        .topology(topology.clone())
        .blockchain(config.build_blockchain())
        .build(config.session.clone())?;

    let mut processes = Vec::new();

    while !topology.nodes.is_empty() {
        let alias = topology
            .nodes
            .values()
            .find(|n| n.trusted_peers.is_empty())
            .map(|n| n.alias.clone())
            .ok_or(Error::CircularTrust)?;

        let spawn_params = config.node_spawn_params(&alias)?;

        processes.push(monitor_controller.spawn_node_custom(spawn_params.verbose(args.verbose))?);

        topology.nodes.remove(&alias);
        topology.nodes.values_mut().for_each(|n| {
            n.trusted_peers.remove(&alias);
        });
    }

    println!("Waiting for Ctrl-C to exit..");

    monitor_controller.monitor_nodes();

    static POOLS: ThreadPool = ThreadPool::new();
    POOLS
        .set_threads(1)
        .expect("could not start statistic thread");

    let rests_and_progress: Vec<(JormungandrRest, ProgressBarController)> = processes
        .iter()
        .map(|x| (x.rest(), x.progress_bar().clone()))
        .collect();

    POOLS.spawn_handle(move || {
        loop {
           run_health_check(&rests_and_progress);
           std::thread::sleep(std::time::Duration::from_secs(2));
        }
    });

    ctrlc::set_handler(move || {
        POOLS
            .set_threads(0)
            .expect("could not stop statistic thread");
        for process in processes.iter_mut() {
            process.finish_monitoring();
        }
        tx.send(()).expect("Could not send signal on channel.")
    })
    .expect("Error setting Ctrl-C handler");

    rx.recv().expect("Could not receive from channel.");
    monitor_controller.finalize();
    Ok(())
}

pub fn run_health_check(monitors: &[(JormungandrRest, ProgressBarController)]) {
    monitors.iter().for_each(|(rest, progress_bar)| {
        let stats = rest.stats();

        if let Err(err) = &stats {
            progress_bar.log_err(format!("cannot connect: {}", err));
            return;
        }
        let stats = stats.unwrap();
        if stats.stats.is_none() {
            progress_bar.log_info(format!("unexpected node state: {:?}", stats.state));
            return;
        }

        let stats = stats.stats.unwrap();

        let fragment_logs = rest.fragment_logs();

        if let Err(err) = &fragment_logs {
            progress_bar.log_err(format!("cannot connect: {}", err));
            return;
        }

        let fragment_logs = fragment_logs.unwrap();

        progress_bar.log_info(format!(
            "tip: {}, chain length: {}, fragments: [in_block: {}, pending: {}, rejected: {}]",
            stats.last_block_hash.unwrap_or_else(|| "genesis".to_string()),
            stats.last_block_height.unwrap_or_else(|| "0".to_string()),
            fragment_logs.values().filter(|x| x.is_in_a_block()).count(),
            fragment_logs.values().filter(|x| x.is_pending()).count(),
            fragment_logs.values().filter(|x| x.is_rejected()).count()
        ));
    })
}