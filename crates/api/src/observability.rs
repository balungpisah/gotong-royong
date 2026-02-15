use std::sync::OnceLock;
use std::time::Duration;

use anyhow::Result;
use axum::http::StatusCode;
use metrics::{counter, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

const HTTP_REQUESTS_TOTAL: &str = "gotong_api_http_requests_total";
const HTTP_REQUEST_DURATION_SECONDS: &str = "gotong_api_http_request_duration_seconds";
const HTTP_REQUEST_ERRORS_TOTAL: &str = "gotong_api_http_errors_total";
const EDGEPOD_FALLBACK_TOTAL: &str = "gotong_api_edgepod_fallback_total";
const EDGEPOD_FALLBACK_UNAVAILABLE_TOTAL: &str = "gotong_api_edgepod_model_unavailable_total";
const CHAT_REALTIME_BRIDGE_EVENTS_TOTAL: &str = "gotong_api_chat_realtime_bridge_events_total";

static METRICS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

pub fn init_metrics() -> Result<()> {
    let handle = PrometheusBuilder::new().install_recorder()?;
    let _ = METRICS_HANDLE.set(handle);
    Ok(())
}

pub fn render_metrics() -> Option<String> {
    METRICS_HANDLE.get().map(PrometheusHandle::render)
}

pub fn register_http_request(method: &str, route: &str, status: StatusCode, elapsed: Duration) {
    let status_code = status.as_u16().to_string();
    let duration_seconds = elapsed.as_secs_f64();
    let result = if status.is_server_error() {
        "error"
    } else {
        "success"
    };

    counter!(
        HTTP_REQUESTS_TOTAL,
        "method" => method.to_string(),
        "route" => route.to_string(),
        "status" => status_code.clone(),
        "result" => result
    )
    .increment(1);

    histogram!(
        HTTP_REQUEST_DURATION_SECONDS,
        "method" => method.to_string(),
        "route" => route.to_string(),
        "status" => status_code
    )
    .record(duration_seconds);

    if status.is_server_error() {
        counter!(
            HTTP_REQUEST_ERRORS_TOTAL,
            "method" => method.to_string(),
            "route" => route.to_string(),
            "status" => status.as_u16().to_string()
        )
        .increment(1);
    }
}

pub fn register_edgepod_fallback(endpoint: &str, reason_code: &str) {
    counter!(
        EDGEPOD_FALLBACK_TOTAL,
        "endpoint" => endpoint.to_string(),
        "reason_code" => reason_code.to_string()
    )
    .increment(1);

    if reason_code == "MODEL_UNAVAILABLE" {
        counter!(
            EDGEPOD_FALLBACK_UNAVAILABLE_TOTAL,
            "endpoint" => endpoint.to_string()
        )
        .increment(1);
    }
}

pub fn register_chat_realtime_bridge_event(event: &str, transport: &str, reason: &str) {
    counter!(
        CHAT_REALTIME_BRIDGE_EVENTS_TOTAL,
        "event" => event.to_string(),
        "transport" => transport.to_string(),
        "reason" => reason.to_string()
    )
    .increment(1);
}
