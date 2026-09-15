use std::time::Duration;

use axum::{
    body::Body,
    extract::MatchedPath,
    http::{header::HeaderName, Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    Router,
};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::Span;

const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

pub(super) fn apply(router: Router) -> Router {
    router
        .layer(PropagateRequestIdLayer::new(X_REQUEST_ID))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(make_request_span)
                .on_request(())
                .on_response(|response: &Response, latency: Duration, _span: &Span| {
                    log_completed(response.status(), latency);
                })
                .on_failure(()),
        )
        .layer(SetRequestIdLayer::new(X_REQUEST_ID, MakeRequestUuid))
        .layer(middleware::from_fn(remove_client_request_id))
}

fn make_request_span(request: &Request<Body>) -> Span {
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .and_then(|value| value.header_value().to_str().ok())
        .unwrap_or("<missing>");
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or("<unmatched>");
    tracing::info_span!(
        "http_request",
        request_id,
        method = %request.method(),
        route,
        external_auth_diagnostic = tracing::field::Empty,
    )
}

async fn remove_client_request_id(mut request: Request<Body>, next: Next) -> Response {
    request.headers_mut().remove(X_REQUEST_ID);
    next.run(request).await
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResponseLogLevel {
    Info,
    Warn,
    Error,
}

fn response_log_level(status: StatusCode) -> ResponseLogLevel {
    if status.is_server_error() {
        ResponseLogLevel::Error
    } else if status.is_client_error() {
        ResponseLogLevel::Warn
    } else {
        ResponseLogLevel::Info
    }
}

fn log_completed(status: StatusCode, latency: Duration) {
    let level = response_log_level(status);
    let status = status.as_u16();
    let latency_ms = u64::try_from(latency.as_millis()).unwrap_or(u64::MAX);
    match level {
        ResponseLogLevel::Info => {
            tracing::info!(status, latency_ms, "request completed");
        }
        ResponseLogLevel::Warn => {
            tracing::warn!(status, latency_ms, "request completed");
        }
        ResponseLogLevel::Error => {
            tracing::error!(status, latency_ms, "request completed");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{self, Write},
        sync::{Arc, Mutex},
    };

    use super::{log_completed, make_request_span, response_log_level, ResponseLogLevel};
    use axum::http::{header, Request, StatusCode};

    #[derive(Clone)]
    struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for CaptureWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0
                .lock()
                .expect("capture writer lock")
                .extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn response_statuses_map_to_the_approved_operational_levels() {
        assert_eq!(response_log_level(StatusCode::OK), ResponseLogLevel::Info);
        assert_eq!(
            response_log_level(StatusCode::TEMPORARY_REDIRECT),
            ResponseLogLevel::Info
        );
        assert_eq!(
            response_log_level(StatusCode::FORBIDDEN),
            ResponseLogLevel::Warn
        );
        assert_eq!(
            response_log_level(StatusCode::INTERNAL_SERVER_ERROR),
            ResponseLogLevel::Error
        );
    }

    #[test]
    fn request_trace_never_records_authorization_body_query_or_cookie_values() {
        let output = Arc::new(Mutex::new(Vec::new()));
        let writer_output = Arc::clone(&output);
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_writer(move || CaptureWriter(Arc::clone(&writer_output)))
            .finish();
        let sentinel = "task10-secret-token-authorization-sentinel";
        let request = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/external/token?secret={sentinel}"))
            .header(header::AUTHORIZATION, format!("Bearer {sentinel}"))
            .header(header::COOKIE, format!("session={sentinel}"))
            .body(axum::body::Body::from(sentinel))
            .expect("sentinel request");

        tracing::subscriber::with_default(subscriber, || {
            let span = make_request_span(&request);
            span.in_scope(|| {
                log_completed(
                    StatusCode::UNAUTHORIZED,
                    std::time::Duration::from_millis(4),
                )
            });
        });

        let output = String::from_utf8(output.lock().expect("capture output lock").clone())
            .expect("trace output is UTF-8");
        assert!(!output.contains(sentinel));
        assert!(output.contains("request completed"));
        assert!(output.contains("status=401"));
    }
}
