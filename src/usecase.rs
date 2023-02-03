use serenity::prelude::TypeMapKey;

use self::members::MembersUseCase;

pub(crate) mod firebase;
pub(crate) mod members;

pub(crate) struct UseCaseContainer<UR> {
    pub(crate) members: MembersUseCase<UR>,
}

impl<UR: Send + Sync + 'static> TypeMapKey for UseCaseContainer<UR> {
    type Value = Self;
}
