use std::{error::Error, future::Future, net::SocketAddr};

use axum::{
    Router,
    body::Body,
    extract::State,
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::get,
};
use opentelemetry::metrics::{Gauge, MeterProvider as _};
use opentelemetry_sdk::{Resource, metrics::SdkMeterProvider};
use prometheus::{Encoder, Registry, TextEncoder};
use tokio::net::TcpListener;

pub async fn serve(
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let telemetry = Telemetry::new()?;
    let listen_addr = SocketAddr::from(([0, 0, 0, 0], 9090));
    let listener = TcpListener::bind(listen_addr).await?;

    println!("observability server listening on http://{listen_addr}");
    println!("  health  /healthz");
    println!("  metrics /metrics");

    axum::serve(listener, router(telemetry.registry.clone()))
        .with_graceful_shutdown(shutdown)
        .await?;

    telemetry.up.record(0, &[]);
    telemetry.provider.shutdown()?;
    Ok(())
}

fn router(registry: Registry) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/metrics", get(metrics))
        .with_state(registry)
}

async fn health() -> &'static str {
    "ok\n"
}

async fn metrics(State(registry): State<Registry>) -> Response {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();

    if encoder.encode(&registry.gather(), &mut buffer).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let mut response = Response::new(Body::from(buffer));
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
    );
    response
}

struct Telemetry {
    registry: Registry,
    provider: SdkMeterProvider,
    up: Gauge<u64>,
}

impl Telemetry {
    fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let registry = Registry::new();
        let exporter = opentelemetry_prometheus::exporter()
            .with_registry(registry.clone())
            .build()?;
        let provider = SdkMeterProvider::builder()
            .with_resource(
                Resource::builder()
                    .with_service_name("homeostat-controller")
                    .build(),
            )
            .with_reader(exporter)
            .build();
        let meter = provider.meter("homeostat-controller");
        let up = meter
            .u64_gauge("homeostat_controller_up")
            .with_description("Whether the Homeostat controller process is running")
            .build();

        up.record(1, &[]);

        Ok(Self {
            registry,
            provider,
            up,
        })
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, extract::State, http::StatusCode};

    use super::{Telemetry, health, metrics};

    #[tokio::test]
    async fn health_reports_ok() {
        assert_eq!(health().await, "ok\n");
    }

    #[tokio::test]
    async fn metrics_use_prometheus_text_format() {
        let telemetry = Telemetry::new().unwrap();

        let response = metrics(State(telemetry.registry.clone())).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()["content-type"],
            "text/plain; version=0.0.4; charset=utf-8"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("homeostat_controller_up"));
        assert!(body.contains("service_name=\"homeostat-controller\""));
    }
}
