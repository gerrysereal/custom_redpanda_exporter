use lazy_static::lazy_static;
use prometheus::{
    IntGaugeVec, IntCounterVec, Opts, Registry,
};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    
    pub static ref REDPANDA_MESSAGES: IntCounterVec = IntCounterVec::new(
        Opts::new("redpanda_messages_total", "Total messages in topics"),
        &["topic"]
    ).unwrap();

    pub static ref REDPANDA_LAG: IntGaugeVec = IntGaugeVec::new(
        Opts::new("redpanda_consumer_lag", "Consumer lag per topic"),
        &["topic", "consumer_group"]
    ).unwrap();
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(REDPANDA_MESSAGES.clone())).unwrap();
    REGISTRY.register(Box::new(REDPANDA_LAG.clone())).unwrap();
}