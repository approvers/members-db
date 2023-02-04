use std::sync::Arc;

use anyhow::Context as _;
use firestore::FirestoreDb;
use tokio::sync::Mutex;

use crate::infra::repository::firestore::{MemberDataRepositoryImpl, OAuth2RepositoryImpl};
use crate::util::safe_env;

use super::members::MembersUseCase;
use super::oauth2::OAuth2UseCase;
use super::UseCaseContainer;

pub(crate) type FirebaseUseCaseContainer =
    UseCaseContainer<MemberDataRepositoryImpl, OAuth2RepositoryImpl>;

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
        oauth2: OAuth2UseCase::new(oauth2_repository),
    }))
}
