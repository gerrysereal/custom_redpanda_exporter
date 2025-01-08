mod metrics;
mod collectors;
mod config;

use prometheus::{Encoder, TextEncoder};
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::convert::Infallible;
use std::net::SocketAddr;

async fn metrics_handler(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let encoder = TextEncoder::new();
    let metric_families = crate::metrics::redpanda::REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", encoder.format_type())
        .body(Body::from(buffer))
        .unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Register metrics
    crate::metrics::redpanda::register_metrics();
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 9102));
    
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(metrics_handler))
    });

    let server = Server::bind(&addr).serve(make_svc);
    println!("Server running on http://{}", addr);

    server.await?;
    Ok(())
}