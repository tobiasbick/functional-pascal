//! Native-only collection state. No FPAS values cross the sampling thread.
use super::data;
use crate::SystemInfoIntrinsic as Op;
use serde_json::{Value as Json, json};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use sysinfo::{Components, Disks, Gpus, Networks, System};

#[derive(Default)]
pub(super) struct Collector {
    cpu: Option<System>,
    memory: Option<System>,
    processes: Option<System>,
    disks: Option<Disks>,
    networks: Option<Networks>,
    components: Option<Components>,
    gpus: Option<Gpus>,
    previous: [Option<Instant>; 12],
}

pub(super) fn timestamp() -> Result<u64, String> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    u64::try_from(elapsed.as_millis()).map_err(|e| e.to_string())
}

pub(super) fn slot<T>(
    slot: &mut Option<T>,
    create: impl FnOnce() -> Result<T, sysinfo::Error>,
) -> Result<&mut T, String> {
    if slot.is_none() {
        *slot = Some(create().map_err(|e| e.to_string())?);
    }
    slot.as_mut()
        .ok_or_else(|| "Host information state was not initialized".into())
}

pub(super) fn section(result: Result<Json, String>) -> Json {
    match result {
        Ok(value) => json!({"Ok": value}),
        Err(reason) => json!({"Error": reason}),
    }
}

impl Collector {
    pub(super) fn query(&mut self, operation: Op) -> Result<Json, String> {
        if operation == Op::Snapshot {
            let mut output = serde_json::Map::new();
            output.insert("StartedAtMilliseconds".into(), timestamp()?.into());
            for (name, operation) in [
                ("System", Op::Info),
                ("Cpu", Op::Cpu),
                ("Memory", Op::Memory),
                ("Disks", Op::Disks),
                ("Networks", Op::Networks),
                ("Components", Op::Components),
                ("Machine", Op::Machine),
                ("Gpus", Op::Gpus),
                ("Processes", Op::InspectProcesses),
                ("Users", Op::ListUsers),
                ("Groups", Op::ListGroups),
            ] {
                output.insert(name.into(), section(self.query(operation)));
            }
            output.insert("FinishedAtMilliseconds".into(), timestamp()?.into());
            return Ok(output.into());
        }
        let sampled_at = timestamp()?;
        let now = Instant::now();
        let index = operation as usize - Op::Info as usize;
        let interval = self.previous[index].map(|last| now.duration_since(last));
        let measured =
            interval.is_some_and(|elapsed| elapsed >= sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        let mut value = match operation {
            Op::Info => data::system(),
            Op::Cpu => {
                let system = slot(&mut self.cpu, System::new)?;
                if interval.is_none() || measured {
                    system.refresh_cpu_all();
                }
                data::cpu(system, measured)
            }
            Op::Memory => {
                let system = slot(&mut self.memory, System::new)?;
                system.refresh_memory();
                data::memory(system)
            }
            Op::Disks => {
                let disks = slot(&mut self.disks, Disks::new)?;
                disks.refresh(true);
                json!({"Disks": data::disks(disks)})
            }
            Op::Networks => {
                let networks = slot(&mut self.networks, Networks::new)?;
                networks.refresh(true);
                json!({"Networks": data::networks(networks)})
            }
            Op::Components => {
                let components = slot(&mut self.components, Components::new)?;
                components.refresh(true);
                json!({"SampledAtMilliseconds": sampled_at, "Components": data::components(components)})
            }
            Op::Machine => data::machine()?,
            Op::Gpus => {
                let gpus = slot(&mut self.gpus, Gpus::new)?;
                gpus.refresh(true);
                json!({"Gpus": data::gpus(gpus, measured)})
            }
            Op::InspectProcesses => {
                let system = slot(&mut self.processes, System::new)?;
                data::processes(system, measured)
            }
            Op::ListUsers => data::users()?,
            Op::ListGroups => data::groups()?,
            Op::Snapshot => unreachable!("handled full collection"),
        };
        if let Some(object) = value.as_object_mut() {
            if matches!(
                operation,
                Op::Cpu | Op::Memory | Op::Disks | Op::Networks | Op::Gpus | Op::InspectProcesses
            ) {
                object.insert("SampledAtMilliseconds".into(), sampled_at.into());
            }
            if matches!(
                operation,
                Op::Cpu | Op::Disks | Op::Networks | Op::Gpus | Op::InspectProcesses
            ) {
                let interval = if operation == Op::Cpu && !measured {
                    None
                } else {
                    interval
                };
                object.insert(
                    "IntervalMilliseconds".into(),
                    json!(interval.map(|d| d.as_millis() as u64)),
                );
            }
        }
        if operation != Op::Cpu || interval.is_none() || measured {
            self.previous[index] = Some(now);
        }
        Ok(value)
    }
}
