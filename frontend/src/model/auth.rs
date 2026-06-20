use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
pub struct Auth {
    pub jwt: String,
    pub user: User,
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct User {
    pub email: String,
    pub username: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub totp_enabled: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
pub struct Signup {
    pub email: String,
    pub username: String,
    pub password: String,
    pub password_repeat: String,
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    pub email_or_username: String,
    pub password: String,
}
