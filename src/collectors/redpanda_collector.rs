 
use rdkafka::config::ClientConfig;
use rdkafka::consumer::BaseConsumer;

pub struct RedpandaCollector {
    client: BaseConsumer, 
}

impl RedpandaCollector {
    pub fn new(brokers: &str) -> Self {
        let client: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .create()
            .expect("Failed to create Redpanda client");

        RedpandaCollector { client }
    }

    pub async fn collect_metrics(&self) {
        self.collect_message_counts().await;
        self.collect_consumer_lag().await;
        self.collect_latency().await;
    }

    async fn collect_message_counts(&self) {
        println!("Collecting message counts");
    }

    async fn collect_consumer_lag(&self) {
        println!("Collecting consumer lag");
    }

    async fn collect_latency(&self) {
        println!("Collecting latency metrics");
    }
}