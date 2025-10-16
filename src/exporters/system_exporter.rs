use std::net::IpAddr;

use super::Exporter;
use opentelemetry::KeyValue;
use sysinfo::{Disks, Networks, System};

pub struct SystemExporter {
    system: System,
    base_attributes: Vec<KeyValue>,
    memory_used: opentelemetry::metrics::Gauge<u64>,
    memory_total: opentelemetry::metrics::Gauge<u64>,
    cpu_usage: opentelemetry::metrics::Gauge<f64>,
    disk_total_space: opentelemetry::metrics::Gauge<u64>,
    disk_used_space: opentelemetry::metrics::Gauge<u64>,
    network_received: opentelemetry::metrics::Gauge<u64>,
    network_transmitted: opentelemetry::metrics::Gauge<u64>,
}

impl SystemExporter {
    pub fn new(base_attributes: Vec<KeyValue>) -> Self {
        let meter = opentelemetry::global::meter("sysinfo");

        let memory_used = meter
            .u64_gauge("system.memory.used")
            .with_description("The amount of memory that is currently used in bytes")
            .build();

        let memory_total = meter
            .u64_gauge("system.memory.total")
            .with_description("The total amount of memory in the system in bytes")
            .build();

        let cpu_usage = meter
            .f64_gauge("system.cpu.usage")
            .with_description("The cpu usage percentage")
            .build();

        let system_info = meter
            .u64_gauge("system.info")
            .with_description("The system information")
            .build();

        let disk_total_space = meter
            .u64_gauge("system.disk.total_space")
            .with_description("The total space for a disk in bytes")
            .build();

        let disk_used_space = meter
            .u64_gauge("system.disk.used_space")
            .with_description("The used space for a disk in bytes")
            .build();

        let network_received = meter
            .u64_gauge("system.network.received")
            .with_description("The total amount of data received by the network interface in bytes")
            .build();

        let network_transmitted = meter
            .u64_gauge("system.network.transmitted")
            .with_description(
                "The total amount of data transmitted by the network interface in bytes",
            )
            .build();

        system_info.record(
            1,
            &[
                base_attributes.clone(),
                vec![KeyValue::new("os", std::env::consts::OS)],
                vec![KeyValue::new("arch", std::env::consts::ARCH)],
                vec![KeyValue::new("os_family", std::env::consts::FAMILY)],
                vec![KeyValue::new("distribution", System::distribution_id())],
                vec![KeyValue::new(
                    "os_version",
                    System::os_version().unwrap_or_default(),
                )],
                vec![KeyValue::new(
                    "long_os_version",
                    System::long_os_version().unwrap_or_default(),
                )],
                vec![KeyValue::new(
                    "kernel_version",
                    System::kernel_version().unwrap_or_default(),
                )],
            ]
            .concat(),
        );

        Self {
            system: System::new_all(),
            base_attributes,
            cpu_usage,
            memory_used,
            memory_total,
            disk_total_space,
            disk_used_space,
            network_transmitted,
            network_received,
        }
    }
}

impl Exporter for SystemExporter {
    fn export(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.system.refresh_all();

        self.memory_used
            .record(self.system.used_memory(), &self.base_attributes);
        self.memory_total
            .record(self.system.total_memory(), &self.base_attributes);

        self.cpu_usage.record(
            (self.system.global_cpu_usage() / self.system.cpus().len() as f32).into(),
            &[
                self.base_attributes.clone(),
                vec![KeyValue::new("cpu", "global")],
            ]
            .concat(),
        );

        for cpu in self.system.cpus() {
            let attributes = vec![KeyValue::new("cpu", cpu.name().to_string())];
            self.cpu_usage.record(
                cpu.cpu_usage().into(),
                &[self.base_attributes.clone(), attributes].concat(),
            );
        }

        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            let total_space = disk.total_space();
            let attributes = vec![
                KeyValue::new(
                    "name".to_string(),
                    disk.name().to_str().unwrap_or("").to_string(),
                ),
                KeyValue::new(
                    "file_system".to_string(),
                    disk.file_system().to_str().unwrap_or("").to_string(),
                ),
                KeyValue::new(
                    "mount_point".to_string(),
                    disk.mount_point().to_str().unwrap_or("").to_string(),
                ),
                KeyValue::new("kind".to_string(), format!("{:?}", disk.kind())),
            ];

            self.disk_used_space.record(
                total_space - disk.available_space(),
                &[self.base_attributes.clone(), attributes.clone()].concat(),
            );
            self.disk_total_space.record(
                total_space,
                &[self.base_attributes.clone(), attributes].concat(),
            );
        }

        let networks = Networks::new_with_refreshed_list();
        for (interface_name, data) in &networks {
            let mut attributes = vec![
                KeyValue::new("interface".to_string(), interface_name.to_string()),
                KeyValue::new("mac_address".to_string(), data.mac_address().to_string()),
            ];

            for ip_network in data.ip_networks() {
                match ip_network.addr {
                    IpAddr::V4(ip) => {
                        attributes.push(KeyValue::new("ipv4".to_string(), ip.to_string()));
                    }
                    IpAddr::V6(ip) => {
                        attributes.push(KeyValue::new("ipv6".to_string(), ip.to_string()));
                    }
                }
            }

            self.network_received.record(
                data.total_received(),
                &[self.base_attributes.clone(), attributes.clone()].concat(),
            );

            self.network_transmitted.record(
                data.total_transmitted(),
                &[self.base_attributes.clone(), attributes.clone()].concat(),
            );
        }

        Ok(())
    }
}
