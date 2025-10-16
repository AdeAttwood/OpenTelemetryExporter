# OpenTelemetry Exporter

A service that collects system metrics and log files, then exports them to OpenTelemetry Protocol (OTLP) endpoints.

## Features

- **System Metrics Export**: Collects and exports system-level metrics (CPU, memory, etc.) using OpenTelemetry.
- **Log File Export**: Monitors specified log files and exports their contents as logs.
- **Service Support**: Can run as a Windows service or Linux systemd service for background operation.
- **Configurable**: Easily configure endpoints, exporters, and more via a YAML config file.

## Installation

### Prebuilt binaries

Binaries can be downloaded from the [releases](https://github.com/AdeAttwood/OpenTelemetryExporter/releases) page

### Install from source

1. Clone the repository:
   ```bash
   git clone https://github.com/AdeAttwood/OpenTelemetryExporter.git
   cd OpenTelemetryExporter
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Place the executable (`target/release/open-telemetry-exporter.exe`) in your desired directory.

## Configuration

Create a `config.yaml` file in the same directory as the executable with the following structure:

```yaml
metrics_endpoint: "https://your-otlp-endpoint.com/v1/metrics"
metrics_token: "your_metrics_token"
logs_endpoint: "https://your-otlp-endpoint.com/v1/logs"
logs_token: "your_logs_token"

exporters:
  - type: System
  - type: Log
    filename: "/path/to/logfile.log"
```

- `metrics_endpoint` and `logs_endpoint`: URLs for OTLP HTTP endpoints.
- `metrics_token` and `logs_token`: Authentication tokens for the endpoints.
- `exporters`: List of exporters to enable.
  - `System`: Exports system metrics.
  - `Log`: Exports logs from files matching the `filename` glob pattern.

## Exporters

### System

The following system metrics are exported:

| Metric Name | Description | Type |
|-------------|-------------|------|
| system.memory.used | The amount of memory that is currently used in bytes | Gauge |
| system.memory.total | The total amount of memory in the system in bytes | Gauge |
| system.cpu.usage | The CPU usage percentage (global and per CPU) | Gauge |
| system.info | System information (OS, architecture, version, etc.) | Gauge |
| system.disk.total_space | The total space for a disk in bytes | Gauge |
| system.disk.used_space | The used space for a disk in bytes | Gauge |
| system.network.received | The total amount of data received by the network interface in bytes | Gauge |
| system.network.transmitted | The total amount of data transmitted by the network interface in bytes | Gauge |

Each metrics comes with various tags. For example the `system.cpu.usage` is tagged with each cpu name. There is also a `global` tag for the cpu that will record the global cpu usage.

`system.disk.*` is tagged with each disk name and `system.network.*` is tagged with the network name.

### Log

The log exporter is very simple. It will take each line in the log file and emit as a log entry. Each log entry will be sent as `Information` severity, it will also have the file path as the `file` attribute.

## Usage

### Running Normally

```bash
./open-telemetry-exporter
```

### Setting up a windows service

1. Install the service (requires administrator privileges):

```bash
sc create "OpenTelemetryExporter" binPath= "C:\Program Files\OpenTelemetryExporter\open-telemetry-exporter.exe --service"
sc start OpenTelemetryExporter
```

### Running as systemd service

1. Copy the [example](./config/exporter.service) systemd service file in to `/etc/systemd/system/opentelemetry-exporter.service`:

2. Reload systemd and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl start opentelemetry-exporter
sudo systemctl enable opentelemetry-exporter
```

3. Check status:

```bash
sudo systemctl status opentelemetry-exporter
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
