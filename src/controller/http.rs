pub(crate) mod api;
pub(crate) mod oauth2;

use std::net::SocketAddr;
use std::str::FromStr as _;
use std::sync::Arc;

use anyhow::Context as _;
use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;

use crate::infra::repository::firestore::{MemberDataRepositoryImpl, OAuth2RepositoryImpl};
use crate::usecase::firebase::FirebaseUseCaseContainer;
use crate::usecase::members::MembersUseCase;
use crate::usecase::oauth2::OAuth2UseCase;

#[tracing::instrument(skip(usecases))]
pub(crate) async fn start_http_server(
    usecases: Arc<FirebaseUseCaseContainer>,
) -> anyhow::Result<()> {
    let state = AppState { usecases };

    let app = Router::new()
        .nest("/oauth2", oauth2::route())
        .with_state(state);

    let addr = SocketAddr::from_str("127.0.0.1:8080").context("could not parse socket address")?;
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("could not serve server")?;

    Ok(())
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

impl FromRef<AppState> for OAuth2UseCase<OAuth2RepositoryImpl> {
    fn from_ref(input: &AppState) -> Self {
        input.usecases.oauth2.clone()
    }
}

impl FromRef<AppState> for MembersUseCase<MemberDataRepositoryImpl> {
    fn from_ref(input: &AppState) -> Self {
        input.usecases.members.clone()
    }
}

struct HttpError(anyhow::Error);

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:?}", self.0);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for HttpError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
