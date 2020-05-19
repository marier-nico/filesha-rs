use serde;

#[derive(serde::Deserialize)]
pub struct RegistrationInfo {
    pub email: String,
    pub display_name: String,
    pub password: String
}

#[derive(serde::Serialize)]
pub struct RegistrationInfoResponse {
    pub email: String,
    pub display_name: String
}