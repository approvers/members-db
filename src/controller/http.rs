pub(crate) mod api;
pub(crate) mod oauth2;

use anyhow::Context as _;
use std::net::SocketAddr;
use std::str::FromStr;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;

pub(crate) async fn start_http_server() -> anyhow::Result<()> {
    let app = Router::new().nest("/oauth2", oauth2::route()?);

    let addr = SocketAddr::from_str("127.0.0.1:8080").context("could not parse socket address")?;
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("could not serve server")?;

    Ok(())
}

struct HttpError(anyhow::Error);

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
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
