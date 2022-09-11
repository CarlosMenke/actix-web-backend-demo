use secrecy::Secret;

#[derive(serde::Deserialize, Clone)]
pub struct Application {
    pub redis_uri: String,
    pub domain: String,
}

pub enum CookieContentSecurity {
    Private,
    Signed,
}
