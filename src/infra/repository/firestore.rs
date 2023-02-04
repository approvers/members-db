mod members;
mod oauth2;

pub(crate) use self::oauth2::OAuth2RepositoryImpl;
pub(crate) use members::MemberDataRepositoryImpl;

use firestore::errors::FirestoreError;

use super::RepositoryError;

impl From<FirestoreError> for RepositoryError {
    fn from(value: FirestoreError) -> Self {
        match value {
            FirestoreError::ErrorInTransaction(err) => Self::TransactionError(Box::new(err)),
            _ => Self::InternalError(Box::new(value)),
        }
    }
}
