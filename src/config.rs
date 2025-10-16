use serde::Deserialize;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct LogExporterConfig {
    pub filename: String,
}

#[derive(Debug, Deserialize)]
pub struct SystemExporterConfig {}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Exporter {
    System(SystemExporterConfig),
    Log(LogExporterConfig),
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub metrics_endpoint: String,
    pub metrics_token: String,
    pub logs_endpoint: String,
    pub logs_token: String,

    pub exporters: Vec<Exporter>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let config: Config = serde_yaml::from_reader(file)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_config_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        write!(file, "{}", content).expect("Failed to write to temp file");
        file
    }

    #[test]
    fn test_load_valid_config_with_system_exporter() {
        let config_content = r#"
metrics_endpoint: "https://example.com/metrics"
metrics_token: "metrics_token_123"
logs_endpoint: "https://example.com/logs"
logs_token: "logs_token_456"
exporters:
  - type: System
"#;

        let temp_file = create_test_config_file(config_content);
        let config = Config::load(temp_file.path()).expect("Failed to load config");

        assert_eq!(config.metrics_endpoint, "https://example.com/metrics");
        assert_eq!(config.metrics_token, "metrics_token_123");
        assert_eq!(config.logs_endpoint, "https://example.com/logs");
        assert_eq!(config.logs_token, "logs_token_456");
        assert_eq!(config.exporters.len(), 1);

        match &config.exporters[0] {
            Exporter::System(_) => {}
            _ => panic!("Expected System exporter"),
        }
    }

    #[test]
    fn test_load_valid_config_with_log_exporter() {
        let config_content = r#"
metrics_endpoint: "https://metrics.local"
metrics_token: "metrics_abc"
logs_endpoint: "https://logs.local"
logs_token: "logs_xyz"
exporters:
  - type: Log
    filename: "/var/log/test.log"
"#;

        let temp_file = create_test_config_file(config_content);
        let config = Config::load(temp_file.path()).expect("Failed to load config");

        assert_eq!(config.metrics_endpoint, "https://metrics.local");
        assert_eq!(config.metrics_token, "metrics_abc");
        assert_eq!(config.logs_endpoint, "https://logs.local");
        assert_eq!(config.logs_token, "logs_xyz");
        assert_eq!(config.exporters.len(), 1);

        match &config.exporters[0] {
            Exporter::Log(log_config) => {
                assert_eq!(log_config.filename, "/var/log/test.log");
            }
            _ => panic!("Expected Log exporter"),
        }
    }

    #[test]
    fn test_load_config_with_multiple_exporters() {
        let config_content = r#"
metrics_endpoint: "https://multi.example.com/metrics"
metrics_token: "multi_metrics_token"
logs_endpoint: "https://multi.example.com/logs"
logs_token: "multi_logs_token"
exporters:
  - type: System
  - type: Log
    filename: "/tmp/app.log"
  - type: Log
    filename: "/tmp/debug.log"
"#;

        let temp_file = create_test_config_file(config_content);
        let config = Config::load(temp_file.path()).expect("Failed to load config");

        assert_eq!(config.exporters.len(), 3);

        match &config.exporters[0] {
            Exporter::System(_) => {}
            _ => panic!("Expected System exporter at index 0"),
        }

        match &config.exporters[1] {
            Exporter::Log(log_config) => {
                assert_eq!(log_config.filename, "/tmp/app.log");
            }
            _ => panic!("Expected Log exporter at index 1"),
        }

        match &config.exporters[2] {
            Exporter::Log(log_config) => {
                assert_eq!(log_config.filename, "/tmp/debug.log");
            }
            _ => panic!("Expected Log exporter at index 2"),
        }
    }

    #[test]
    fn test_load_config_with_empty_exporters() {
        let config_content = r#"
metrics_endpoint: "https://empty.example.com/metrics"
metrics_token: "empty_metrics_token"
logs_endpoint: "https://empty.example.com/logs"
logs_token: "empty_logs_token"
exporters: []
"#;

        let temp_file = create_test_config_file(config_content);
        let config = Config::load(temp_file.path()).expect("Failed to load config");

        assert_eq!(config.exporters.len(), 0);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = Config::load("nonexistent_file.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_yaml() {
        let invalid_yaml = r#"
metrics_endpoint: "https://example.com/metrics"
metrics_token: "token"
logs_endpoint: "https://example.com/logs"
logs_token: "token"
exporters:
  - type: InvalidType
    invalid_field: value
"#;

        let temp_file = create_test_config_file(invalid_yaml);
        let result = Config::load(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_missing_required_fields() {
        let incomplete_config = r#"
metrics_endpoint: "https://example.com/metrics"
# Missing other required fields
exporters: []
"#;

        let temp_file = create_test_config_file(incomplete_config);
        let result = Config::load(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_malformed_yaml() {
        let malformed_yaml = r#"
metrics_endpoint: "https://example.com/metrics
# Missing closing quote above
metrics_token: "token"
"#;

        let temp_file = create_test_config_file(malformed_yaml);
        let result = Config::load(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_log_exporter_config_debug() {
        let log_config = LogExporterConfig {
            filename: "/test/path.log".to_string(),
        };

        let debug_string = format!("{:?}", log_config);
        assert!(debug_string.contains("LogExporterConfig"));
        assert!(debug_string.contains("/test/path.log"));
    }

    #[test]
    fn test_system_exporter_config_debug() {
        let system_config = SystemExporterConfig {};

        let debug_string = format!("{:?}", system_config);
        assert!(debug_string.contains("SystemExporterConfig"));
    }

    #[test]
    fn test_config_debug() {
        let config = Config {
            metrics_endpoint: "https://test.com/metrics".to_string(),
            metrics_token: "test_token".to_string(),
            logs_endpoint: "https://test.com/logs".to_string(),
            logs_token: "test_logs_token".to_string(),
            exporters: vec![
                Exporter::System(SystemExporterConfig {}),
                Exporter::Log(LogExporterConfig {
                    filename: "/test.log".to_string(),
                }),
            ],
        };

        let debug_string = format!("{:?}", config);
        assert!(debug_string.contains("Config"));
        assert!(debug_string.contains("https://test.com/metrics"));
        assert!(debug_string.contains("test_token"));
    }
}
