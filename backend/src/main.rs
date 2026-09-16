use chrono::Utc;
use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use x_fly_api::{
    application::{
        api_client::ApiClientManagement,
        cancellation::{CancellationService, RefundDispatcher},
        external_analytics::ExternalAnalyticsService,
        external_auth::{ApiClientCredentialService, ExternalAuthService},
        external_flights::ExternalFlightService,
        flight::FlightManagement,
        staff_auth::StaffAuthService,
        use_cases::PaymentApplication,
    },
    config::AppConfig,
    domain::cancellation::SystemClock,
    infrastructure::{
        database::{
            verify_database_ready, SqlxAnalyticsRepository, SqlxApiClientRepository,
            SqlxExternalAnalyticsRepository, SqlxExternalAuthRepository, SqlxFlightRepository,
            SqlxSeatHoldRepository, SqlxStaffAuthRepository,
        },
        diagnostics::classify_sqlx_error,
        external_auth_crypto::HmacExternalCredentialCrypto,
        http::build_router,
        password::Argon2PasswordService,
        payment::{
            stripe::StripePaymentGateway, MockBitcoinPaymentGateway, UnavailableCardPaymentGateway,
        },
        refund::{stripe::StripeRefundGateway, MockBitcoinRefundGateway},
    },
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "x_fly_api=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env()?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;
    verify_database_ready(&pool).await?;

    let repository = Arc::new(SqlxSeatHoldRepository::new(pool));
    let analytics = Arc::new(SqlxAnalyticsRepository::new(repository.pool().clone()));
    let external_analytics = ExternalAnalyticsService::new(Arc::new(
        SqlxExternalAnalyticsRepository::new(repository.pool().clone()),
    ));
    let staff_auth = StaffAuthService::new_with_max_concurrent_password_verifications(
        Arc::new(SqlxStaffAuthRepository::new(repository.pool().clone())),
        Argon2PasswordService::default(),
        std::time::Duration::from_secs(60 * 60),
        config.staff_auth_max_concurrent_verifications,
    )?;
    let payments = match config.stripe_secret_key.clone() {
        Some(key) => {
            let stripe = Arc::new(StripePaymentGateway::new(key));
            PaymentApplication::new(
                repository.clone(),
                stripe.clone(),
                Arc::new(MockBitcoinPaymentGateway),
            )
            .with_stripe_provider(stripe)
        }
        None => PaymentApplication::new(
            repository.clone(),
            Arc::new(UnavailableCardPaymentGateway),
            Arc::new(MockBitcoinPaymentGateway),
        ),
    };
    let cancellation_service = CancellationService::new(repository.clone(), Arc::new(SystemClock));
    let refund_dispatcher = RefundDispatcher::new(
        repository.clone(),
        config
            .stripe_secret_key
            .clone()
            .map(|key| Arc::new(StripeRefundGateway::new(key)) as _),
        Arc::new(MockBitcoinRefundGateway),
    );
    let state = AppState::new(
        repository.clone(),
        repository.clone(),
        repository.clone(),
        repository.clone(),
        config.seat_hold_ttl,
        config.secure_cookies,
        config.frontend_origin,
    )
    .with_payments(payments)
    .with_stripe_webhook_secret(config.stripe_webhook_secret)
    .with_tickets(repository.clone(), config.ticket_qr_signing_secret)
    .with_cancellations(cancellation_service);
    let flights = FlightManagement::new(Arc::new(SqlxFlightRepository::new(
        repository.pool().clone(),
    )));
    let external_auth_repository =
        Arc::new(SqlxExternalAuthRepository::new(repository.pool().clone()));
    let external_auth_crypto = Arc::new(HmacExternalCredentialCrypto::from_pepper(
        config.external_api_credential_pepper.clone(),
    ));
    let api_client_credentials = ApiClientCredentialService::new(
        external_auth_repository.clone(),
        external_auth_crypto.clone(),
    );
    let external_auth = ExternalAuthService::new(external_auth_repository, external_auth_crypto);
    let state = state
        .with_staff_auth(staff_auth)
        .with_analytics(analytics)
        .with_external_analytics(external_analytics)
        .with_flights(flights.clone())
        .with_external_flights(ExternalFlightService::new(flights))
        .with_booking_management(repository.clone())
        .with_ticket_operations(repository.clone())
        .with_api_clients(ApiClientManagement::new(Arc::new(
            SqlxApiClientRepository::new(repository.pool().clone()),
        )))
        .with_external_auth(api_client_credentials, external_auth);
    let state =
        state.with_manage_bookings(repository.clone(), config.manage_booking_signing_secret);
    let listener = tokio::net::TcpListener::bind(config.bind_address).await?;
    tracing::info!(address = %config.bind_address, "X-Fly API listening");
    let refund_worker = tokio::spawn(async move {
        loop {
            if let Err(error) = refund_dispatcher.dispatch_once(Utc::now()).await {
                tracing::warn!(
                    component = "refund_worker",
                    operation = "dispatch",
                    stage = error.stage(),
                    refund_job_id = ?error.job_id(),
                    error_category = %classify_sqlx_error(error.source()),
                    "refund dispatch attempt failed"
                );
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });
    axum::serve(listener, build_router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    refund_worker.abort();
    let _ = refund_worker.await;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install terminate signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
