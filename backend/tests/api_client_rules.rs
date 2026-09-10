use x_fly_api::domain::api_client::{
    ApiClientScope, ApiClientStatus, ApiClientValidationError, CreateApiClientCommand,
    UpdateApiClientCommand,
};

#[test]
fn create_normalizes_safe_metadata_and_deduplicates_typed_scopes() {
    let validated = CreateApiClientCommand {
        name: "  Approved   Analytics  ".to_owned(),
        description: Some("  Aggregate reporting only.  ".to_owned()),
        status: ApiClientStatus::Active,
        allowed_scopes: vec![ApiClientScope::AnalyticsRead, ApiClientScope::AnalyticsRead],
    }
    .validate()
    .unwrap();

    assert_eq!(validated.name, "Approved Analytics");
    assert_eq!(
        validated.description.as_deref(),
        Some("Aggregate reporting only.")
    );
    assert_eq!(
        validated.allowed_scopes,
        vec![ApiClientScope::AnalyticsRead]
    );
}

#[test]
fn metadata_and_active_scope_invariants_are_strict() {
    for command in [
        CreateApiClientCommand {
            name: "   ".to_owned(),
            description: None,
            status: ApiClientStatus::Suspended,
            allowed_scopes: vec![],
        },
        CreateApiClientCommand {
            name: "A".repeat(101),
            description: None,
            status: ApiClientStatus::Suspended,
            allowed_scopes: vec![],
        },
        CreateApiClientCommand {
            name: "Valid".to_owned(),
            description: Some("D".repeat(501)),
            status: ApiClientStatus::Suspended,
            allowed_scopes: vec![],
        },
    ] {
        assert_eq!(command.validate(), Err(ApiClientValidationError::Metadata));
    }

    assert_eq!(
        CreateApiClientCommand {
            name: "No scope".to_owned(),
            description: None,
            status: ApiClientStatus::Active,
            allowed_scopes: vec![],
        }
        .validate(),
        Err(ApiClientValidationError::ActiveRequiresScope)
    );

    assert!(CreateApiClientCommand {
        name: "Prepared integration".to_owned(),
        description: None,
        status: ApiClientStatus::Suspended,
        allowed_scopes: vec![],
    }
    .validate()
    .is_ok());
}

#[test]
fn update_uses_the_same_invariants_and_requires_a_positive_version() {
    assert_eq!(
        UpdateApiClientCommand {
            name: "Approved".to_owned(),
            description: None,
            allowed_scopes: vec![ApiClientScope::FlightsRead],
            version: 0,
        }
        .validate(ApiClientStatus::Active),
        Err(ApiClientValidationError::Version)
    );
}

#[test]
fn scope_and_status_parsing_never_accepts_arbitrary_values() {
    assert_eq!(
        ApiClientScope::parse("flights:read"),
        Some(ApiClientScope::FlightsRead)
    );
    assert_eq!(
        ApiClientScope::parse("analytics:read"),
        Some(ApiClientScope::AnalyticsRead)
    );
    assert_eq!(ApiClientScope::parse("passengers:read"), None);
    assert_eq!(ApiClientScope::parse("custom:anything"), None);
    assert_eq!(
        ApiClientStatus::parse("ACTIVE"),
        Some(ApiClientStatus::Active)
    );
    assert_eq!(ApiClientStatus::parse("active"), None);
}

#[test]
fn scope_codes_round_trip_and_normalize_by_canonical_code() {
    for (scope, code) in [
        (ApiClientScope::AnalyticsRead, "analytics:read"),
        (ApiClientScope::FlightsRead, "flights:read"),
    ] {
        assert_eq!(serde_json::to_value(scope).unwrap(), code);
        assert_eq!(
            serde_json::from_str::<ApiClientScope>(&format!("\"{code}\"")).unwrap(),
            scope
        );
    }

    let validated = CreateApiClientCommand {
        name: "Canonical ordering".to_owned(),
        description: None,
        status: ApiClientStatus::Active,
        allowed_scopes: vec![ApiClientScope::FlightsRead, ApiClientScope::AnalyticsRead],
    }
    .validate()
    .unwrap();
    assert_eq!(
        validated.allowed_scopes,
        vec![ApiClientScope::AnalyticsRead, ApiClientScope::FlightsRead]
    );
}

#[test]
fn revoked_is_terminal_and_active_requires_an_allowed_scope() {
    assert!(ApiClientStatus::Active.can_transition_to(ApiClientStatus::Suspended, true));
    assert!(ApiClientStatus::Suspended.can_transition_to(ApiClientStatus::Active, true));
    assert!(!ApiClientStatus::Suspended.can_transition_to(ApiClientStatus::Active, false));
    assert!(ApiClientStatus::Active.can_transition_to(ApiClientStatus::Revoked, true));
    assert!(ApiClientStatus::Suspended.can_transition_to(ApiClientStatus::Revoked, false));
    assert!(!ApiClientStatus::Revoked.can_transition_to(ApiClientStatus::Active, true));
    assert!(!ApiClientStatus::Revoked.can_transition_to(ApiClientStatus::Suspended, true));
    assert!(!ApiClientStatus::Active.can_transition_to(ApiClientStatus::Active, true));
}
