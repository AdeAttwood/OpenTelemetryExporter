use opentelemetry_otlp::WithExportConfig;
use opentelemetry_otlp::WithHttpConfig;
use opentelemetry_otlp::{LogExporter, MetricExporter, Protocol};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::{logs::SdkLoggerProvider, metrics::SdkMeterProvider};

pub fn build_metrics_exporter(endpoint: &str, token: &str) -> SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::Grpc)
        .with_endpoint(endpoint)
        .with_headers(
            [("Authorization".to_string(), format!("Bearer {}", token))]
                .into_iter()
                .collect(),
        )
        .build()
        .unwrap();

    let resource = Resource::builder().with_service_name("exporter").build();

    SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(resource)
        .build()
}

pub fn build_log_exporter(endpoint: &str, token: &str) -> SdkLoggerProvider {
    let exporter = LogExporter::builder()
        .with_http()
        .with_protocol(Protocol::Grpc)
        .with_endpoint(endpoint)
        .with_headers(
            [("Authorization".to_string(), format!("Bearer {}", token))]
                .into_iter()
                .collect(),
        )
        .build()
        .expect("Failed to create log exporter");

    let resource = Resource::builder().with_service_name("exporter").build();

    SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build()
}
