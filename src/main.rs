use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use prometheus::{register_gauge, Encoder, Gauge, Opts, TextEncoder};
use reqwest;
use std::convert::Infallible;
use std::error::Error;
use tokio;
use lazy_static::lazy_static;

// Inisialisasi metrics Redpanda
lazy_static! {
    static ref REDPANDA_MEMORY_FREE: Gauge = register_gauge!(
        Opts::new("redpanda_memory_free_bytes", "Redpanda free memory in bytes")
    ).unwrap();
    
    static ref REDPANDA_MEMORY_TOTAL: Gauge = register_gauge!(
        Opts::new("redpanda_memory_total_bytes", "Redpanda total memory in bytes")
    ).unwrap();
    
    static ref REDPANDA_MEMORY_USED: Gauge = register_gauge!(
        Opts::new("redpanda_memory_used_bytes", "Redpanda used memory in bytes")
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

    async fn collect_redpanda_metrics(&self) -> Result<(), Box<dyn Error>> {
        let response = self.client
            .get(&self.redpanda_url)
            .send()
            .await?
            .text()
            .await?;

        for line in response.lines() {
            if line.starts_with("memory_free_bytes") {
                if let Some(value) = line.split_whitespace().last() {
                    if let Ok(bytes) = value.parse::<f64>() {
                        REDPANDA_MEMORY_FREE.set(bytes);
                    }
                }
            } else if line.starts_with("memory_total_bytes") {
                if let Some(value) = line.split_whitespace().last() {
                    if let Ok(bytes) = value.parse::<f64>() {
                        REDPANDA_MEMORY_TOTAL.set(bytes);
                    }
                }
            } else if line.starts_with("memory_used_bytes") {
                if let Some(value) = line.split_whitespace().last() {
                    if let Ok(bytes) = value.parse::<f64>() {
                        REDPANDA_MEMORY_USED.set(bytes);
                    }
                }
            }
        }

        Ok(())
    }

    async fn collect_metrics(&self) -> Result<String, Box<dyn Error>> {
        self.collect_redpanda_metrics().await?;

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