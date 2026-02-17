use std::sync::OnceLock;

use anyhow::Result;
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

const JOBS_PROCESSED_TOTAL: &str = "gotong_worker_jobs_processed_total";
const JOBS_PROCESSING_DURATION_MS: &str = "gotong_worker_job_processing_duration_ms";
const QUEUE_READY_GAUGE: &str = "gotong_worker_queue_ready_total";
const QUEUE_DELAYED_GAUGE: &str = "gotong_worker_queue_delayed_total";
const QUEUE_PROCESSING_GAUGE: &str = "gotong_worker_queue_processing_total";
const QUEUE_LAG_GAUGE: &str = "gotong_worker_queue_lag_ms";
const WEBHOOK_DELIVERY_TOTAL: &str = "gotong_worker_webhook_delivery_total";
const WEBHOOK_DELIVERY_DURATION_MS: &str = "gotong_worker_webhook_delivery_duration_ms";
const WEBHOOK_DLQ_DEPTH_GAUGE: &str = "gotong_worker_webhook_dead_letter_total";

static METRICS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

pub fn init_metrics() -> Result<()> {
    let handle = PrometheusBuilder::new().install_recorder()?;
    let _ = METRICS_HANDLE.set(handle);
    Ok(())
}

pub fn _render_metrics() -> Option<String> {
    METRICS_HANDLE.get().map(PrometheusHandle::render)
}

pub fn register_job_processed(job_type: &str, result: &str, duration_ms: f64) {
    counter!(
        JOBS_PROCESSED_TOTAL,
        "job_type" => job_type.to_string(),
        "result" => result.to_string()
    )
    .increment(1);

    histogram!(
        JOBS_PROCESSING_DURATION_MS,
        "job_type" => job_type.to_string()
    )
    .record(duration_ms);
}

pub fn set_queue_depth_gauge(ready: u64, delayed: u64, processing: u64) {
    gauge!(QUEUE_READY_GAUGE).set(ready as f64);
    gauge!(QUEUE_DELAYED_GAUGE).set(delayed as f64);
    gauge!(QUEUE_PROCESSING_GAUGE).set(processing as f64);
}

pub fn set_queue_lag_ms(lag_ms: i64) {
    gauge!(QUEUE_LAG_GAUGE).set(lag_ms.max(0) as f64);
}

pub fn register_webhook_delivery(result: &str, status_code: Option<u16>, duration_ms: f64) {
    let status_code = status_code
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string());

    counter!(
        WEBHOOK_DELIVERY_TOTAL,
        "result" => result.to_string(),
        "status_code" => status_code.clone()
    )
    .increment(1);

    histogram!(
        WEBHOOK_DELIVERY_DURATION_MS,
        "result" => result.to_string(),
        "status_code" => status_code
    )
    .record(duration_ms.max(0.0));
}

pub fn set_webhook_dead_letter_depth(depth: u64) {
    gauge!(WEBHOOK_DLQ_DEPTH_GAUGE).set(depth as f64);
}
