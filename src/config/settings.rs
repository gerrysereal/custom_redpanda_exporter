use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub redpanda: RedpandaConfig,
    pub exporter: ExporterConfig,
}

#[derive(Debug, Deserialize)]
pub struct RedpandaConfig {
    pub brokers: Vec<String>,
    pub topics: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExporterConfig {
    pub port: u16,
    pub metrics_path: String,
}