use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct UserData {
    pub discord_user_id: String,
    pub display_name: Option<String>,
    pub oauth2: UserOAuth2Data,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct UserOAuth2Data {
    pub access_token: String,
    pub refresh_token: String,
}
