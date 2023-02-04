use std::sync::Arc;

use anyhow::Context as _;
use firestore::FirestoreDb;
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use tokio::sync::Mutex;

use crate::infra::repository::firestore::{MemberDataRepositoryImpl, OAuth2RepositoryImpl};
use crate::util::safe_env;

use super::members::MembersUseCase;
use super::oauth2::OAuth2UseCase;
use super::UseCaseContainer;

pub(crate) type FirebaseUseCaseContainer =
    UseCaseContainer<MemberDataRepositoryImpl, OAuth2RepositoryImpl>;

fn oauth2_client() -> anyhow::Result<BasicClient> {
    let client_id = safe_env("OAUTH2_CLIENT_ID")?;
    let client_secret = safe_env("OAUTH2_CLIENT_SECRET")?;
    let redirect_url = "http://localhost:8080/oauth2/discord/callback".to_string();
    let auth_url = "https://discord.com/api/oauth2/authorize?response_type=code".to_string();
    let token_url = "https://discord.com/api/oauth2/token".to_string();

    Ok(BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(auth_url).context("could not parse oauth2 auth-url")?,
        Some(TokenUrl::new(token_url).context("could not parse oauth2 token-url")?),
    )
    .set_redirect_uri(
        RedirectUrl::new(redirect_url).context("could not parse oauth2 redirect-url")?,
    ))
}

pub(crate) async fn get_firebase_usecases() -> anyhow::Result<Arc<FirebaseUseCaseContainer>> {
    let firestore_db = Arc::new(Mutex::new(
        FirestoreDb::new(safe_env("GOOGLE_PROJECT_ID")?)
            .await
            .context("could not initialize firestore client")?,
    ));

    let user_data_repository =
        MemberDataRepositoryImpl::new(Arc::clone(&firestore_db), "members_data");
    let oauth2_repository = OAuth2RepositoryImpl::new(firestore_db, "oauth2_data");

    Ok(Arc::new(UseCaseContainer {
        members: MembersUseCase::new(user_data_repository),
        oauth2: OAuth2UseCase::new(oauth2_client()?, oauth2_repository),
    }))
}
