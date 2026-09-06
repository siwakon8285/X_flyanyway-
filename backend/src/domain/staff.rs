use std::{collections::BTreeSet, fmt, str::FromStr};

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaffEmail(String);

impl StaffEmail {
    pub fn parse(value: &str) -> Result<Self, StaffIdentityError> {
        let normalized = value.trim().to_lowercase();
        let valid = normalized.len() <= 254
            && normalized.bytes().filter(|byte| *byte == b'@').count() == 1
            && normalized.split_once('@').is_some_and(|(local, domain)| {
                !local.is_empty()
                    && !domain.is_empty()
                    && domain.contains('.')
                    && !domain.starts_with('.')
                    && !domain.ends_with('.')
                    && !normalized.chars().any(char::is_whitespace)
            });
        valid
            .then_some(Self(normalized))
            .ok_or(StaffIdentityError::InvalidEmail)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RoleCode {
    Executive,
    FlightManager,
    BookingOperations,
    TicketPassengerOperations,
    BaggageStaff,
    ApiAdmin,
    SystemAdmin,
}

impl RoleCode {
    pub const ALL: [Self; 7] = [
        Self::Executive,
        Self::FlightManager,
        Self::BookingOperations,
        Self::TicketPassengerOperations,
        Self::BaggageStaff,
        Self::ApiAdmin,
        Self::SystemAdmin,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Executive => "EXECUTIVE",
            Self::FlightManager => "FLIGHT_MANAGER",
            Self::BookingOperations => "BOOKING_OPERATIONS",
            Self::TicketPassengerOperations => "TICKET_PASSENGER_OPERATIONS",
            Self::BaggageStaff => "BAGGAGE_STAFF",
            Self::ApiAdmin => "API_ADMIN",
            Self::SystemAdmin => "SYSTEM_ADMIN",
        }
    }
}

impl FromStr for RoleCode {
    type Err = StaffIdentityError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|role| role.as_str() == value)
            .ok_or(StaffIdentityError::UnknownRole)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PermissionCode {
    DashboardRead,
    AnalyticsRead,
    ReportsRead,
    FlightsRead,
    FlightsWrite,
    BookingsRead,
    BookingsManage,
    TicketsRead,
    TicketsPrint,
    PassengersRead,
    BookingsReadLimited,
    PassengersReadLimited,
    BaggageContextRead,
    ApiClientsRead,
    ApiClientsManage,
    StaffRead,
    StaffManage,
    RolesRead,
    RolesManage,
}

impl PermissionCode {
    pub const ALL: [Self; 19] = [
        Self::DashboardRead,
        Self::AnalyticsRead,
        Self::ReportsRead,
        Self::FlightsRead,
        Self::FlightsWrite,
        Self::BookingsRead,
        Self::BookingsManage,
        Self::TicketsRead,
        Self::TicketsPrint,
        Self::PassengersRead,
        Self::BookingsReadLimited,
        Self::PassengersReadLimited,
        Self::BaggageContextRead,
        Self::ApiClientsRead,
        Self::ApiClientsManage,
        Self::StaffRead,
        Self::StaffManage,
        Self::RolesRead,
        Self::RolesManage,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DashboardRead => "dashboard:read",
            Self::AnalyticsRead => "analytics:read",
            Self::ReportsRead => "reports:read",
            Self::FlightsRead => "flights:read",
            Self::FlightsWrite => "flights:write",
            Self::BookingsRead => "bookings:read",
            Self::BookingsManage => "bookings:manage",
            Self::TicketsRead => "tickets:read",
            Self::TicketsPrint => "tickets:print",
            Self::PassengersRead => "passengers:read",
            Self::BookingsReadLimited => "bookings:read_limited",
            Self::PassengersReadLimited => "passengers:read_limited",
            Self::BaggageContextRead => "baggage_context:read",
            Self::ApiClientsRead => "api_clients:read",
            Self::ApiClientsManage => "api_clients:manage",
            Self::StaffRead => "staff:read",
            Self::StaffManage => "staff:manage",
            Self::RolesRead => "roles:read",
            Self::RolesManage => "roles:manage",
        }
    }
}

impl FromStr for PermissionCode {
    type Err = StaffIdentityError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|permission| permission.as_str() == value)
            .ok_or(StaffIdentityError::UnknownPermission)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaffPrincipal {
    staff_user_id: Uuid,
    session_id: Uuid,
    email: String,
    roles: BTreeSet<RoleCode>,
    permissions: BTreeSet<PermissionCode>,
    expires_at: DateTime<Utc>,
}

impl StaffPrincipal {
    pub fn new(
        staff_user_id: Uuid,
        session_id: Uuid,
        email: String,
        roles: Vec<RoleCode>,
        permissions: Vec<PermissionCode>,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            staff_user_id,
            session_id,
            email,
            roles: roles.into_iter().collect(),
            permissions: permissions.into_iter().collect(),
            expires_at,
        }
    }

    pub fn staff_user_id(&self) -> Uuid {
        self.staff_user_id
    }
    pub fn session_id(&self) -> Uuid {
        self.session_id
    }
    pub fn email(&self) -> &str {
        &self.email
    }
    pub fn roles(&self) -> &BTreeSet<RoleCode> {
        &self.roles
    }
    pub fn permissions(&self) -> &BTreeSet<PermissionCode> {
        &self.permissions
    }
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }
    pub fn has_role(&self, role: RoleCode) -> bool {
        self.roles.contains(&role)
    }
    pub fn can(&self, permission: PermissionCode) -> bool {
        self.permissions.contains(&permission)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaffIdentityError {
    InvalidEmail,
    UnknownRole,
    UnknownPermission,
}

impl fmt::Display for StaffIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid staff identity value")
    }
}

impl std::error::Error for StaffIdentityError {}

#[derive(Clone, Debug)]
pub struct StaffCredential {
    pub id: Uuid,
    pub password_hash: String,
    pub active: bool,
}
