use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use reqwest;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::error::Error;
use tokio;

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

    async fn collect_metrics(&self) -> Result<String, Box<dyn Error>> {
        let response = self
            .client
            .get(&self.redpanda_url)
            .send()
            .await?
            .text()
            .await?;
        Ok(response)
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

    let make_svc = make_service_fn(move |_conn| {
        let collector = collector.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| handle_metrics(collector.clone(), req)))
        }
    });

    println!("Starting metrics server on http://0.0.0.0:9102");
    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}