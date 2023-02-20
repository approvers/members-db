pub(crate) mod api;
pub(crate) mod oauth2;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context as _;
use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;
use tokio::signal;

use crate::infra::repository::firestore::{MemberDataRepositoryImpl, OAuth2RepositoryImpl};
use crate::service::members::MembersService;
use crate::usecase::firebase::FirebaseUseCaseContainer;
use crate::usecase::members::MembersUseCase;
use crate::usecase::oauth2::OAuth2UseCase;
use crate::util::safe_env;

#[tracing::instrument(skip(usecases))]
pub(crate) async fn start_http_server(
    usecases: Arc<FirebaseUseCaseContainer>,
) -> anyhow::Result<()> {
    let state = AppState { usecases };

    let app = Router::new()
        .nest("/oauth2", oauth2::route())
        .nest("/api/v1", api::route())
        .with_state(state);

    let port = safe_env("PORT")?.parse::<u16>()?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("could not serve server")?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        #[allow(clippy::expect_used)]
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        #[allow(clippy::expect_used)]
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, staring gracing shutdown")
}

#[derive(Clone)]
pub(crate) struct AppState {
    usecases: Arc<FirebaseUseCaseContainer>,
}

impl FromRef<AppState> for Arc<FirebaseUseCaseContainer> {
    fn from_ref(input: &AppState) -> Self {
        Arc::clone(&input.usecases)
    }
}

impl FromRef<AppState> for OAuth2UseCase<MemberDataRepositoryImpl, OAuth2RepositoryImpl> {
    fn from_ref(input: &AppState) -> Self {
        input.usecases.oauth2.clone()
    }
}

impl FromRef<AppState> for MembersUseCase<MemberDataRepositoryImpl> {
    fn from_ref(input: &AppState) -> Self {
        input.usecases.members.clone()
    }
}

impl FromRef<AppState> for MembersService<MemberDataRepositoryImpl, OAuth2RepositoryImpl> {
    fn from_ref(input: &AppState) -> Self {
        input.usecases.members_service.clone()
    }
}

struct HttpError(StatusCode, anyhow::Error);

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        (self.0, format!("something went wrong: {}", self.1)).into_response()
    }
}

impl<E> From<E> for HttpError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, err.into())
    }
}
