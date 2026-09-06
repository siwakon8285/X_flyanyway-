use std::str::FromStr;

use x_fly_api::{
    application::staff_auth::{ProvisionMode, StaffAdminCommand},
    domain::staff::{PermissionCode, RoleCode, StaffEmail, StaffPrincipal},
    infrastructure::password::Argon2PasswordService,
};

#[test]
fn staff_email_is_normalized_without_accepting_invalid_identity_shapes() {
    let email = StaffEmail::parse("  Flight.Manager@X-Fly.Internal  ").unwrap();
    assert_eq!(email.as_str(), "flight.manager@x-fly.internal");

    for invalid in [
        "",
        "staff",
        "@x-fly.internal",
        "staff@",
        "a b@x.test",
        "staff@x@x.test",
        "staff@.test",
    ] {
        assert!(StaffEmail::parse(invalid).is_err(), "accepted {invalid:?}");
    }
}

#[test]
fn canonical_role_and_permission_codes_are_strict() {
    assert_eq!(
        RoleCode::from_str("FLIGHT_MANAGER").unwrap(),
        RoleCode::FlightManager
    );
    assert!(RoleCode::from_str("SYSTEM_ADMINISTRATOR").is_err());
    assert_eq!(
        PermissionCode::from_str("flights:write").unwrap(),
        PermissionCode::FlightsWrite
    );
    assert!(PermissionCode::from_str("admin:all").is_err());
}

#[test]
fn multi_role_principal_uses_the_union_without_implicit_superuser_access() {
    let principal = StaffPrincipal::new(
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        "staff@x-fly.internal".to_owned(),
        vec![RoleCode::SystemAdmin, RoleCode::ApiAdmin],
        vec![
            PermissionCode::StaffManage,
            PermissionCode::RolesManage,
            PermissionCode::ApiClientsManage,
            PermissionCode::StaffManage,
        ],
        chrono::Utc::now() + chrono::Duration::minutes(60),
    );

    assert!(principal.has_role(RoleCode::SystemAdmin));
    assert!(principal.has_role(RoleCode::ApiAdmin));
    assert!(principal.can(PermissionCode::StaffManage));
    assert!(principal.can(PermissionCode::ApiClientsManage));
    assert!(!principal.can(PermissionCode::FlightsWrite));
    assert_eq!(principal.permissions().len(), 3);
}

#[test]
fn argon2id_hashes_are_salted_verified_and_upgrade_aware() {
    let service = Argon2PasswordService::default();
    let first = service.hash("correct horse battery staple").unwrap();
    let second = service.hash("correct horse battery staple").unwrap();

    assert!(first.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));
    assert_ne!(first, second);
    assert!(service.verify("correct horse battery staple", &first));
    assert!(!service.verify("wrong password", &first));
    assert!(!service.needs_rehash(&first));
}

#[test]
fn staff_admin_cli_requires_an_explicit_mode_email_and_canonical_roles() {
    let command = StaffAdminCommand::parse([
        "create",
        "--email",
        "flight@x-fly.internal",
        "--role",
        "FLIGHT_MANAGER",
    ])
    .unwrap();
    assert_eq!(command.mode(), ProvisionMode::Create);
    assert_eq!(command.email(), "flight@x-fly.internal");
    assert_eq!(command.roles(), &[RoleCode::FlightManager]);

    for invalid in [
        vec!["create", "--email", "staff@x.test"],
        vec!["bootstrap", "--role", "SYSTEM_ADMIN"],
        vec!["create", "--email", "staff@x.test", "--password", "secret"],
        vec!["create", "--email", "staff@x.test", "--role", "ADMIN"],
    ] {
        assert!(StaffAdminCommand::parse(invalid).is_err());
    }
}
