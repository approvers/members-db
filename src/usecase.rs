use std::sync::Arc;

use serenity::prelude::TypeMapKey;

use crate::service::members::MembersService;

use self::members::MembersUseCase;
use self::oauth2::OAuth2UseCase;

pub(crate) mod firebase;
pub(crate) mod members;
pub(crate) mod oauth2;

#[derive(Clone)]
pub(crate) struct UseCaseContainer<MR: Clone, OR: Clone> {
    pub(crate) members: MembersUseCase<MR>,
    pub(crate) oauth2: OAuth2UseCase<MR, OR>,
    pub(crate) members_service: MembersService<MR, OR>,
}

impl<UR, OR> TypeMapKey for UseCaseContainer<UR, OR>
where
    UR: Clone + Send + Sync + 'static,
    OR: Clone + Send + Sync + 'static,
{
    type Value = Arc<Self>;
}
