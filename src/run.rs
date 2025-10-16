use crate::config::{Config, Exporter as ConfigExporter};
use crate::exporters::{Exporter, LogExporter, SystemExporter};
use crate::otlp_init::{build_log_exporter, build_metrics_exporter};

use glob::glob;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use sysinfo::System;

use opentelemetry::KeyValue;
use opentelemetry::logs::{LogRecord, Logger, LoggerProvider};

pub async fn run_app(running: Arc<AtomicBool>) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = std::env::current_exe()?
        .parent()
        .unwrap()
        .join("config.yaml");
    let config = Config::load(config_path)?;

    let log_logger = build_log_exporter(&config.logs_endpoint, &config.logs_token);
    let logger = log_logger.logger("exporter");

    let mut base_attributes: Vec<KeyValue> = vec![];
    if let Some(host_name) = System::host_name() {
        base_attributes.push(KeyValue::new("hostname", host_name));
    }

    let meter_provider = build_metrics_exporter(&config.metrics_endpoint, &config.metrics_token);
    opentelemetry::global::set_meter_provider(meter_provider);

    let mut exporters: Vec<Box<dyn Exporter>> = vec![];

    for exporter in &config.exporters {
        match exporter {
            ConfigExporter::System(_) => {
                crate::log::macros::info!(logger, "Adding system metrics exporter");
                let system_exporter = SystemExporter::new(base_attributes.clone());
                exporters.push(Box::new(system_exporter));
            }
            ConfigExporter::Log(log_config) => {
                for entry in glob(&log_config.filename).expect("Failed to read glob pattern") {
                    match entry {
                        Ok(path) => {
                            crate::log::macros::info!(
                                logger,
                                format!("Adding new log file for {:?}", path)
                            );

                            let log_exporter = LogExporter::new(
                                path,
                                log_logger.clone(),
                                base_attributes.clone(),
                            )?;
                            exporters.push(Box::new(log_exporter));
                        }
                        Err(e) => crate::log::macros::error!(logger, format!("{:?}", e)),
                    }
                }
            }
        }
    }

    while running.load(Ordering::SeqCst) {
        for exporter in exporters.iter_mut() {
            if let Err(e) = exporter.export() {
                crate::log::macros::error!(logger, format!("Exporter error: {}", e));
            }
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    Ok(())
}

#[cfg(windows)]
pub mod windows_service_support {
    use super::run_app;
    use std::ffi::OsString;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use std::thread;
    use std::time::Duration;
    use tokio::runtime::Builder;

    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher,
    };

    define_windows_service!(ffi_service_main, my_service_main);

    pub fn run_as_service() -> Result<(), windows_service::Error> {
        service_dispatcher::start("Exporter", ffi_service_main)
    }

    fn my_service_main(_args: Vec<OsString>) {
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        let status_handle =
            service_control_handler::register("Exporter", move |control| match control {
                ServiceControl::Stop => {
                    r.store(false, Ordering::SeqCst);
                    ServiceControlHandlerResult::NoError
                }
                _ => ServiceControlHandlerResult::NotImplemented,
            })
            .unwrap();

        status_handle
            .set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Running,
                controls_accepted: ServiceControlAccept::STOP,
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })
            .unwrap();

        let handle = thread::spawn(move || {
            Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(run_app(running))
                .unwrap();
        });

        handle.join().unwrap();

        status_handle
            .set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Stopped,
                controls_accepted: ServiceControlAccept::empty(),
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })
            .unwrap();
    }
}
