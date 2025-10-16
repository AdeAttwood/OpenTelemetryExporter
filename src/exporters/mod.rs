pub mod log_exporter;
pub mod system_exporter;

pub use log_exporter::LogExporter;
pub use system_exporter::SystemExporter;

pub trait Exporter {
    fn export(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}
