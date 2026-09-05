use chrono::Utc;
use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use x_fly_api::{
    application::use_cases::PaymentApplication,
    config::AppConfig,
    infrastructure::{
        database::{prepare_database, SqlxSeatHoldRepository},
        email::resend::ResendEmailDeliveryGateway,
        http::build_router,
        payment::{
            stripe::StripePaymentGateway, MockBitcoinPaymentGateway, UnavailableCardPaymentGateway,
        },
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
    prepare_database(&pool).await?;

    let repository = Arc::new(SqlxSeatHoldRepository::new(pool));
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
    .with_tickets(repository.clone(), config.ticket_qr_signing_secret);
    let state =
        state.with_manage_bookings(repository.clone(), config.manage_booking_signing_secret);
    let listener = tokio::net::TcpListener::bind(config.bind_address).await?;
    tracing::info!(address = %config.bind_address, "X-Fly API listening");
    let email_worker = if config.email_transport.eq_ignore_ascii_case("resend") {
        let gateway = Arc::new(ResendEmailDeliveryGateway::new(
            config
                .resend_api_key
                .clone()
                .expect("validated Resend API key"),
            config.email_from.clone().expect("validated email sender"),
        )?);
        let service =
            x_fly_api::application::booking_confirmation::BookingConfirmationEmailService::new(
                repository.clone(),
                repository.clone(),
                gateway,
                config.public_site_origin.clone(),
            );
        let worker_service = service.clone();
        Some(tokio::spawn(async move {
            loop {
                let _ = worker_service.dispatch_once(Utc::now()).await;
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }))
    } else {
        None
    };
    axum::serve(listener, build_router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    if let Some(worker) = email_worker {
        worker.abort();
        let _ = worker.await;
    }
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
