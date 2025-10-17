use axum_macros::FromRequestParts;
use axum_extra::{TypedHeader, headers::{Authorization, authorization::Bearer}};

#[derive(FromRequestParts)]
pub struct AuthHeader {
    #[from_request(via(TypedHeader))]
    auth: Authorization<Bearer>,
}

impl AuthHeader {
    pub fn token(&self) -> &str {
        self.auth.token()
    }
}
