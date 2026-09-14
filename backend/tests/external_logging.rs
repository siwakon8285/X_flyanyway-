use std::{fs, path::PathBuf};

use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
};
use http_body_util::BodyExt;
use serde_json::Value;
use uuid::Uuid;
use x_fly_api::infrastructure::http::external::{ExternalErrorCode, ExternalErrorEnvelope};

fn workflow_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.github/workflows/ci.yml")
}

#[test]
fn ci_generates_and_exports_an_ephemeral_external_pepper_without_printing_it() {
    let workflow = fs::read_to_string(workflow_path()).expect("CI workflow");
    assert!(workflow.contains(r#"external_pepper="$(openssl rand -hex 32)""#));
    assert!(workflow.contains(r#"echo "::add-mask::$external_pepper""#));
    assert!(
        workflow.contains("printf 'EXTERNAL_API_CREDENTIAL_PEPPER_V1=%s\\n' \"$external_pepper\"")
    );
    assert!(!workflow.contains("printf 'EXTERNAL_API_CREDENTIAL_PEPPER_V1=001122"));
    let pepper_generation = workflow
        .find(r#"external_pepper="$(openssl rand -hex 32)""#)
        .expect("pepper generation step");
    let first_runtime_config_command = workflow
        .find("cargo test --locked")
        .expect("backend test command");
    assert!(pepper_generation < first_runtime_config_command);
}

#[tokio::test]
async fn external_error_envelope_contains_only_safe_public_fields() {
    let request_id = Uuid::new_v4();
    let response =
        ExternalErrorEnvelope::new(ExternalErrorCode::ExternalAuthenticationFailed, request_id)
            .into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(response.headers()[header::WWW_AUTHENTICATE], "Bearer");
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("error body")
        .to_bytes();
    let body: Value = serde_json::from_slice(&bytes).expect("JSON error envelope");
    let keys = body["error"]
        .as_object()
        .expect("error object")
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(keys, vec!["code", "message", "requestId"]);
    assert_eq!(body["error"]["requestId"], request_id.to_string());
    assert_eq!(body["error"]["code"], "EXTERNAL_AUTHENTICATION_FAILED");
    assert!(!body.to_string().contains("credential"));
    assert!(!body.to_string().contains("token"));
    assert!(!body.to_string().contains("uuid"));
}

#[tokio::test]
async fn scope_denial_is_generic_and_does_not_challenge_bearer_authentication() {
    let response =
        ExternalErrorEnvelope::new(ExternalErrorCode::ExternalScopeDenied, Uuid::new_v4())
            .into_response();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(response.headers().get(header::WWW_AUTHENTICATE).is_none());
    let body = response
        .into_body()
        .collect()
        .await
        .expect("error body")
        .to_bytes();
    let body: Value = serde_json::from_slice(&body).expect("JSON error envelope");
    assert_eq!(body["error"]["code"], "EXTERNAL_SCOPE_DENIED");
    assert_eq!(body["error"].as_object().unwrap().len(), 3);
}
