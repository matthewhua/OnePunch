use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use tracing::info;

pub fn init_metrics(addr: SocketAddr) {
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(addr)
        .install()
        .expect("failed to install Prometheus recorder");
        
    info!("Metrics exporter listening on http://{}", addr);
    
    // 初始化一些静态指标
    metrics::describe_gauge!(
        "sector_marching_troops",
        "Number of active marching troops in a sector"
    );
    metrics::describe_counter!(
        "sector_messages_processed",
        "Total number of messages processed by a sector"
    );
}

pub mod world_metrics {
    use metrics::{gauge, counter};

    pub fn record_marching_troops(sector_id: i32, count: usize) {
        metrics::gauge!("sector_marching_troops", "sector" => sector_id.to_string()).set(count as f64);
    }

    pub fn inc_messages_processed(sector_id: i32) {
        metrics::counter!("sector_messages_processed", "sector" => sector_id.to_string()).increment(1);
    }
}
