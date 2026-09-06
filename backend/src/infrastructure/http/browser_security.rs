use axum::http::{header, HeaderMap};

pub(super) fn browser_mutation_is_trusted(headers: &HeaderMap, frontend_origin: &str) -> bool {
    headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        == Some(frontend_origin)
        && headers
            .get("x-x-fly-csrf")
            .and_then(|value| value.to_str().ok())
            == Some("1")
}
