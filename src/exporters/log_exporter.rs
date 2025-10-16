use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;

use super::Exporter;
use opentelemetry::KeyValue;
use opentelemetry::logs::{LogRecord, Logger, LoggerProvider, Severity};
use opentelemetry_sdk::logs::SdkLoggerProvider;

pub struct LogExporter {
    offset: u64,
    path: PathBuf,
    base_attributes: Vec<KeyValue>,
    logger_provider: SdkLoggerProvider,
}

impl LogExporter {
    pub fn new(
        path: PathBuf,
        logger_provider: SdkLoggerProvider,
        base_attributes: Vec<KeyValue>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(&path)?;
        let offset = file.seek(SeekFrom::End(0))?;

        Ok(Self {
            path,
            offset,
            base_attributes,
            logger_provider,
        })
    }
}

impl Exporter for LogExporter {
    fn export(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(&self.path)?;

        // The file may have been truncated or rotated, now we want to read from the start.
        if file.metadata()?.len() < self.offset {
            self.offset = 0;
        }

        file.seek(SeekFrom::Start(self.offset))?;

        let mut reader = BufReader::new(&file);
        let mut line = String::new();

        let logger = self.logger_provider.logger("log_exporter");
        while reader.read_line(&mut line)? > 0 {
            let content = line.trim().to_string();
            if !content.is_empty() {
                let mut log = logger.create_log_record();
                log.set_severity_number(Severity::Info);
                log.set_body(content.into());

                for attribute in self.base_attributes.iter() {
                    log.add_attribute(
                        attribute.key.as_str().to_string(),
                        attribute.value.as_str().to_string(),
                    );
                }

                log.add_attribute("file", self.path.display().to_string());

                logger.emit(log);
            }

            line.clear();
        }

        self.offset = file.stream_position()?;

        Ok(())
    }
}
