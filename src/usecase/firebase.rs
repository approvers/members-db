use anyhow::Context as _;
use firestore::FirestoreDb;

use crate::infra::repository::firestore::UserDataRepositoryImpl;
use crate::util::safe_env;

use super::members::MembersUseCase;
use super::UseCaseContainer;

pub(crate) type FirebaseUseCaseContainer = UseCaseContainer<UserDataRepositoryImpl>;

pub(crate) async fn get_firebase_usecases() -> anyhow::Result<FirebaseUseCaseContainer> {
    let firestore_db = FirestoreDb::new(safe_env("GOOGLE_PROJECT_ID")?)
        .await
        .context("could not initialize firestore client")?;

    let user_data_repository = UserDataRepositoryImpl::new(firestore_db, "user_data");

    Ok(UseCaseContainer {
        members: MembersUseCase::new(user_data_repository),
    })
}
