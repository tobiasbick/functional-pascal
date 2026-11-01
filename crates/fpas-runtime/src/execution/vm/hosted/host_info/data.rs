//! sysinfo field mappings. Native strings remain owned until task-side conversion.
use super::collector::section;
use serde_json::{Value as Json, json};
use sysinfo::{Components, Disks, Gpus, Groups, Networks, System, Users};

fn reading<T: serde::Serialize>(value: Result<T, sysinfo::Error>) -> Json {
    section(value.map(|v| json!(v)).map_err(|e| e.to_string()))
}

pub(super) fn system() -> Json {
    let load = System::load_average().map(|v| {
        json!({
            "OneMinute": v.one, "FiveMinutes": v.five, "FifteenMinutes": v.fifteen,
        })
    });
    json!({
        "Name": reading(System::name()), "KernelVersion": reading(System::kernel_version()),
        "OsVersion": reading(System::os_version()), "LongOsVersion": reading(System::long_os_version()),
        "HostName": reading(System::host_name()), "Architecture": System::cpu_arch(),
        "DistributionId": System::distribution_id(), "DistributionLike": System::distribution_id_like(),
        "UptimeSeconds": reading(System::uptime()), "BootTimeSeconds": reading(System::boot_time()),
        "LoadAverage": reading(load), "OpenFilesLimit": reading(System::open_files_limit()),
    })
}

pub(super) fn cpu(system: &System, measured: bool) -> Json {
    json!({
        "PhysicalCores": reading(System::physical_core_count()),
        "UsagePercent": measured.then(|| system.global_cpu_usage()),
        "Cpus": system.cpus().iter().map(|cpu| json!({
            "Name": cpu.name(), "Vendor": cpu.vendor_id(), "Brand": cpu.brand(),
            "FrequencyMHz": cpu.frequency(), "UsagePercent": measured.then(|| cpu.usage()),
        })).collect::<Vec<_>>(),
    })
}

pub(super) fn cgroup(value: sysinfo::CGroupLimits) -> Json {
    json!({"TotalBytes": value.total_memory, "FreeBytes": value.free_memory,
        "FreeSwapBytes": value.free_swap, "ResidentBytes": value.rss})
}

pub(super) fn memory(system: &System) -> Json {
    json!({
        "TotalBytes": system.total_memory(), "FreeBytes": system.free_memory(),
        "AvailableBytes": system.available_memory(), "UsedBytes": system.used_memory(),
        "TotalSwapBytes": system.total_swap(), "FreeSwapBytes": system.free_swap(),
        "UsedSwapBytes": system.used_swap(), "Cgroup": system.cgroup_limits().map(cgroup),
    })
}

pub(super) fn disks(disks: &Disks) -> Vec<Json> {
    let mut disks = disks.iter().map(|disk| {
        let io = disk.usage();
        json!({
            "Id": disk.id().map(|v| v.to_string()), "Name": disk.name().to_string_lossy(), "Kind": disk.kind().to_string(),
            "FileSystem": disk.file_system().to_string_lossy(), "MountPoint": disk.mount_point().to_string_lossy(),
            "TotalBytes": disk.total_space(), "AvailableBytes": disk.available_space(),
            "Removable": disk.is_removable(), "ReadOnly": disk.is_read_only(),
            "ReadBytes": io.read_bytes, "WrittenBytes": io.written_bytes,
            "TotalReadBytes": io.total_read_bytes, "TotalWrittenBytes": io.total_written_bytes,
        })
    }).collect::<Vec<_>>();
    disks.sort_by(|a, b| a["MountPoint"].as_str().cmp(&b["MountPoint"].as_str()));
    disks
}

pub(super) fn networks(networks: &Networks) -> Vec<Json> {
    let mut networks = networks.iter().map(|(name, n)| json!({
        "Name": name.to_string_lossy(), "MacAddress": n.mac_address().to_string(),
        "IpNetworks": n.ip_networks().iter().map(ToString::to_string).collect::<Vec<_>>(),
        "Mtu": n.mtu(), "State": format!("{:?}", n.operational_state()),
        "TransmitBitsPerSecond": n.transmit_link_speed(), "ReceiveBitsPerSecond": n.receive_link_speed(),
        "ReceivedBytes": n.received(), "TransmittedBytes": n.transmitted(),
        "TotalReceivedBytes": n.total_received(), "TotalTransmittedBytes": n.total_transmitted(),
        "ReceivedPackets": n.packets_received(), "TransmittedPackets": n.packets_transmitted(),
        "TotalReceivedPackets": n.total_packets_received(), "TotalTransmittedPackets": n.total_packets_transmitted(),
        "ReceiveErrors": n.errors_on_received(), "TransmitErrors": n.errors_on_transmitted(),
        "TotalReceiveErrors": n.total_errors_on_received(), "TotalTransmitErrors": n.total_errors_on_transmitted(),
    })).collect::<Vec<_>>();
    networks.sort_by(|a, b| a["Name"].as_str().cmp(&b["Name"].as_str()));
    networks
}

pub(super) fn components(components: &Components) -> Json {
    json!(
        components
            .iter()
            .map(|c| json!({
                "Id": c.id(), "Label": c.label(), "TemperatureCelsius": c.temperature(),
                "MaximumCelsius": c.max(), "CriticalCelsius": c.critical(),
            }))
            .collect::<Vec<_>>()
    )
}

pub(super) fn machine() -> Result<Json, String> {
    use sysinfo::{Motherboard, Product};
    let board = Motherboard::new().map(|board| {
        json!({
            "Name": board.name(), "Vendor": board.vendor_name(), "Version": board.version(),
            "Serial": board.serial_number(), "AssetTag": board.asset_tag(),
        })
    });
    Ok(json!({
        "ProductName": reading(Product::name()), "ProductFamily": reading(Product::family()),
        "ProductVendor": reading(Product::vendor_name()), "ProductVersion": reading(Product::version()),
        "ProductSerial": reading(Product::serial_number()), "ProductSku": reading(Product::stock_keeping_unit()),
        "ProductUuid": reading(Product::uuid()), "Motherboard": reading(board),
    }))
}

pub(super) fn gpus(gpus: &Gpus, measured: bool) -> Vec<Json> {
    gpus.iter()
        .map(|gpu| {
            json!({
                "PciAddress": gpu.pci().to_string(), "Vendor": gpu.vendor(), "Model": gpu.model(),
                "UsagePercent": measured.then(|| gpu.usage()).flatten(),
                "TotalMemoryBytes": gpu.total_memory(), "UsedMemoryBytes": gpu.used_memory(),
            })
        })
        .collect()
}

pub(super) fn processes(system: &mut System, measured: bool) -> Json {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate};
    let previous = system
        .processes()
        .iter()
        .map(|(pid, p)| (*pid, p.start_time()))
        .collect::<std::collections::HashMap<_, _>>();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::everything(),
    );
    let mut processes = system.processes().values().map(|p| {
        let io = p.disk_usage();
        let measured = measured && previous.get(&p.pid()) == Some(&p.start_time());
        let mut tasks = p.tasks().map(|tasks| tasks.iter().map(|id| id.as_u32()).collect::<Vec<_>>());
        if let Some(tasks) = &mut tasks { tasks.sort_unstable(); }
        json!({
            "Pid": p.pid().as_u32(), "ParentPid": p.parent().map(|id| id.as_u32()),
            "Name": p.name().to_string_lossy(), "Status": p.status().to_string(),
            "Executable": p.exe().map(|v| v.to_string_lossy()), "WorkingDirectory": p.cwd().map(|v| v.to_string_lossy()),
            "RootDirectory": p.root().map(|v| v.to_string_lossy()),
            "Arguments": p.cmd().iter().map(|v| v.to_string_lossy()).collect::<Vec<_>>(),
            "Environment": p.environ().iter().map(|v| v.to_string_lossy()).collect::<Vec<_>>(),
            "MemoryBytes": p.memory(), "VirtualMemoryBytes": p.virtual_memory(),
            "StartTimeSeconds": p.start_time(), "RunTimeSeconds": p.run_time(),
            "CpuUsagePercent": measured.then(|| p.cpu_usage()), "CpuTimeMilliseconds": p.accumulated_cpu_time(),
            "ReadBytes": io.read_bytes, "WrittenBytes": io.written_bytes,
            "TotalReadBytes": io.total_read_bytes, "TotalWrittenBytes": io.total_written_bytes,
            "UserId": p.user_id().map(|v| v.to_string()), "EffectiveUserId": p.effective_user_id().map(|v| v.to_string()),
            "GroupId": p.group_id().map(|v| v.to_string()), "EffectiveGroupId": p.effective_group_id().map(|v| v.to_string()),
            "SessionId": p.session_id().map(|id| id.as_u32()), "Tasks": tasks,
            "ThreadKind": p.thread_kind().map(|v| format!("{v:?}")),
            "OpenFiles": p.open_files(), "OpenFilesLimit": p.open_files_limit(),
            "GpuUsagePercent": p.gpu_usage(), "GpuMemoryBytes": p.gpu_memory(), "Cgroup": p.cgroup_limits().map(cgroup),
        })
    }).collect::<Vec<_>>();
    processes.sort_by_key(|p| p["Pid"].as_u64());
    json!({"Processes": processes})
}

fn group(group: &sysinfo::Group) -> Json {
    json!({"Id": group.id().to_string(), "Name": group.name()})
}

pub(super) fn users() -> Result<Json, String> {
    let users = Users::new_with_refreshed_list().map_err(|e| e.to_string())?;
    let mut users = users
        .iter()
        .map(|u| {
            json!({
                "Id": u.id().to_string(), "GroupId": u.group_id().to_string(), "Name": u.name(),
                "Groups": u.groups().iter().map(group).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    users.sort_by(|a, b| a["Id"].as_str().cmp(&b["Id"].as_str()));
    Ok(json!(users))
}

pub(super) fn groups() -> Result<Json, String> {
    let groups = Groups::new_with_refreshed_list().map_err(|e| e.to_string())?;
    let mut groups = groups.iter().map(group).collect::<Vec<_>>();
    groups.sort_by(|a, b| a["Id"].as_str().cmp(&b["Id"].as_str()));
    Ok(json!(groups))
}
