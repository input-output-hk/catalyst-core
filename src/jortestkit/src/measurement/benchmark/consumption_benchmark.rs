use crate::measurement::{
    attribute::{Consumption, NamedProcess},
    marker::ResourcesUsage,
    thresholds::Thresholds,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use sysinfo::AsU32;
use sysinfo::{ProcessExt, SystemExt};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsumptionBenchmarkError {
    #[error("couldn't find process {0}")]
    NoProcessWitId(NamedProcess),
}

#[derive(Clone)]
pub struct ConsumptionBenchmarkDef {
    name: String,
    thresholds: Option<Thresholds<Consumption>>,
    pids: Vec<NamedProcess>,
}

impl ConsumptionBenchmarkDef {
    pub fn new(name: String) -> Self {
        ConsumptionBenchmarkDef {
            name,
            pids: Vec::new(),
            thresholds: None,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn bare_metal_stake_pool_consumption_target(&mut self) -> &mut Self {
        self.thresholds = Some(Thresholds::<Consumption>::new_consumption(
            ResourcesUsage::new(10, 160_000, 20_000_000),
        ));
        self
    }

    pub fn target(&mut self, target: ResourcesUsage) -> &mut Self {
        self.thresholds = Some(Thresholds::<Consumption>::new_consumption(target));
        self
    }

    pub fn no_target(&mut self) -> &mut Self {
        self.thresholds = None;
        self
    }

    pub fn for_process<S: Into<String>>(&mut self, name: S, pid: usize) -> &mut Self {
        self.pids.push(NamedProcess::new(name.into(), pid));
        self
    }

    pub fn for_processes(&mut self, processes: Vec<NamedProcess>) -> &mut Self {
        self.pids.extend(processes);
        self
    }

    pub fn thresholds(&self) -> Option<&Thresholds<Consumption>> {
        self.thresholds.as_ref()
    }

    pub fn start(&self) -> ConsumptionBenchmarkRun {
        ConsumptionBenchmarkRun {
            definition: self.clone(),
            markers: self.pids.iter().map(|x| (x.clone(), vec![])).collect(),
        }
    }

    pub fn start_async(&self, interval: std::time::Duration) -> ConsumptionBenchmarkRunAsync {
        let (stop_signal, mut rx) = tokio::sync::oneshot::channel::<()>();
        let benchmark = Arc::new(Mutex::new(self.start()));
        let benchmark_clone = Arc::clone(&benchmark);

        let handle = std::thread::spawn(move || loop {
            if rx.try_recv().is_ok() {
                break;
            } else {
                benchmark_clone.lock().unwrap().snapshot().unwrap();
                std::thread::sleep(interval);
            }
        });
        ConsumptionBenchmarkRunAsync {
            stop_signal,
            benchmark: Arc::clone(&benchmark),
            handle,
        }
    }
}

pub struct ConsumptionBenchmarkRunAsync {
    stop_signal: tokio::sync::oneshot::Sender<()>,
    handle: JoinHandle<()>,
    benchmark: Arc<Mutex<ConsumptionBenchmarkRun>>,
}

impl ConsumptionBenchmarkRunAsync {
    pub fn stop(self) -> ConsumptionBenchmarkFinish {
        self.stop_signal
            .send(())
            .expect("cannot stop memory consumption benchmark thread");
        self.handle.join().unwrap();
        let benchmark = self.benchmark.lock().unwrap().clone();

        benchmark.stop()
    }
}

#[derive(Clone)]
pub struct ConsumptionBenchmarkRun {
    definition: ConsumptionBenchmarkDef,
    markers: HashMap<NamedProcess, Vec<ResourcesUsage>>,
}

impl ConsumptionBenchmarkRun {
    pub fn snapshot(&mut self) -> Result<(), ConsumptionBenchmarkError> {
        let mut system = sysinfo::System::new();
        system.refresh_processes();

        for (named_process, resources) in self.markers.iter_mut() {
            let (_, process) = system
                .get_processes()
                .iter()
                .find(|(pid, _)| (named_process.id() as u32) == pid.as_u32())
                .ok_or_else(|| ConsumptionBenchmarkError::NoProcessWitId(named_process.clone()))?;

            let marker = ResourcesUsage::new(
                process.cpu_usage() as u32,
                process.memory() as u32,
                process.virtual_memory() as u32,
            );

            resources.push(marker);
        }
        Ok(())
    }

    pub fn exception(self, info: String) -> ConsumptionBenchmarkFinish {
        println!("Test finished prematurely, due to: {}", info);
        self.stop()
    }

    pub fn stop(self) -> ConsumptionBenchmarkFinish {
        match self.definition.thresholds() {
            Some(_thresholds) => ConsumptionBenchmarkFinish {
                definition: self.definition.clone(),
                consumptions: self
                    .markers
                    .iter()
                    .map(|(name, data)| (name.clone(), Consumption::new(data.clone())))
                    .collect(),
            },
            None => ConsumptionBenchmarkFinish {
                definition: self.definition.clone(),
                consumptions: self
                    .markers
                    .iter()
                    .map(|(name, data)| (name.clone(), Consumption::new(data.clone())))
                    .collect(),
            },
        }
    }
}

pub struct ConsumptionBenchmarkFinish {
    definition: ConsumptionBenchmarkDef,
    consumptions: HashMap<NamedProcess, Consumption>,
}

impl ConsumptionBenchmarkFinish {
    pub fn print(&self) {
        for (named_process, consumption) in self.consumptions.iter() {
            self.print_single(named_process, consumption)
        }
    }

    fn print_single(&self, named_process: &NamedProcess, consumption: &Consumption) {
        match self.definition.thresholds() {
            Some(thresholds) => println!(
                "Measurement: {}_{}. Result: {}. Actual: {} Thresholds: {}",
                self.definition.name(),
                named_process.name(),
                consumption.against(thresholds),
                consumption,
                thresholds
            ),
            None => println!(
                "Measurement: {}_{}. Value: {}",
                self.definition.name(),
                named_process.name(),
                consumption
            ),
        }
    }
}
