use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use prometheus::{Encoder, TextEncoder, register_gauge, Gauge, Opts};
use reqwest;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::error::Error;
use std::fs;
use tokio;
use lazy_static::lazy_static;

// Inisialisasi metrics
lazy_static! {
    static ref MEMORY_TOTAL: Gauge = register_gauge!(
        Opts::new("memory_total_bytes", "Total memory in bytes")
    ).unwrap();
    
    static ref MEMORY_USED: Gauge = register_gauge!(
        Opts::new("memory_used_bytes", "Used memory in bytes")
    ).unwrap();
    
    static ref MEMORY_FREE: Gauge = register_gauge!(
        Opts::new("memory_free_bytes", "Free memory in bytes")
    ).unwrap();
    
    static ref CPU_USAGE: Gauge = register_gauge!(
        Opts::new("cpu_usage_percent", "CPU usage in percent")
    ).unwrap();
    
    static ref DISK_USAGE: Gauge = register_gauge!(
        Opts::new("disk_usage_percent", "Disk usage in percent")
    ).unwrap();
}

#[derive(Clone)]
struct MetricsCollector {
    redpanda_url: String,
    client: reqwest::Client,
}

impl MetricsCollector {
    fn new(redpanda_url: String) -> Self {
        MetricsCollector {
            redpanda_url,
            client: reqwest::Client::new(),
        }
    }

    fn collect_memory_metrics(&self) -> Result<(), Box<dyn Error>> {
        let meminfo = fs::read_to_string("/proc/meminfo")?;
        let mut total = 0.0;
        let mut free = 0.0;
        let mut available = 0.0;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total = line.split_whitespace()
                    .nth(1)
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0) * 1024.0; // Convert to bytes
            } else if line.starts_with("MemFree:") {
                free = line.split_whitespace()
                    .nth(1)
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0) * 1024.0;
            } else if line.starts_with("MemAvailable:") {
                available = line.split_whitespace()
                    .nth(1)
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0) * 1024.0;
            }
        }

        let used = total - available;
        
        MEMORY_TOTAL.set(total);
        MEMORY_USED.set(used);
        MEMORY_FREE.set(available);

        Ok(())
    }

    fn collect_cpu_metrics(&self) -> Result<(), Box<dyn Error>> {
        let stat = fs::read_to_string("/proc/stat")?;
        if let Some(cpu_line) = stat.lines().next() {
            let values: Vec<f64> = cpu_line
                .split_whitespace()
                .skip(1)
                .map(|v| v.parse::<f64>().unwrap_or(0.0))
                .collect();

            if values.len() >= 4 {
                let idle = values[3];
                let total: f64 = values.iter().sum();
                let usage = 100.0 * (1.0 - idle / total);
                CPU_USAGE.set(usage);
            }
        }
        Ok(())
    }

    async fn collect_metrics(&self) -> Result<String, Box<dyn Error>> {
        // Collect system metrics
        self.collect_memory_metrics()?;
        self.collect_cpu_metrics()?;

        // Get Prometheus metrics
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer)?;
        
        Ok(String::from_utf8(buffer)?)
    }
}

async fn handle_metrics(
    collector: MetricsCollector,
    _req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    match collector.collect_metrics().await {
        Ok(metrics) => Ok(Response::new(Body::from(metrics))),
        Err(e) => {
            eprintln!("Error collecting metrics: {}", e);
            Ok(Response::new(Body::from(format!(
                "# Error collecting metrics: {}",
                e
            ))))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = ([0, 0, 0, 0], 9102).into();
    let redpanda_url = "http://172.16.192.110:9644/metrics".to_string();
    let collector = MetricsCollector::new(redpanda_url);

    println!("Starting metrics server on http://0.0.0.0:9102");
    
    let make_svc = make_service_fn(move |_conn| {
        let collector = collector.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| handle_metrics(collector.clone(), req)))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}
