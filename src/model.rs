use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct MemberDataRow {
    pub discord_user_id: String,
    pub display_name: Option<String>,
    pub oauth2: MemberOAuth2Data,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct MemberOAuth2Data {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CsrfTokenData {
    pub pkce_verifier: String,
    #[serde(with = "firestore::serialize_as_timestamp")]
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct MemberListRow {
    pub discord_user_id: String,
    pub display_name: Option<String>,
    pub twitter: Vec<String>,
    pub github: Vec<String>,
    pub role: Option<RoleInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct RoleInfo {
    pub name: String,
    pub color: String,
}
