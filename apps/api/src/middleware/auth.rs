
use axum::{extract::FromRequestParts, http::{request::Parts, StatusCode}};

#[derive(Clone, Debug)]
pub struct AgentContext {
    pub user_id: String,
    pub workspace_id: String,
    pub node_url: String,
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AgentContext where S: Send + Sync {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth = parts.headers.get("authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "missing authorization".into()))?;
        let token = auth.strip_prefix("Bearer ").ok_or((StatusCode::UNAUTHORIZED, "invalid authorization scheme".into()))?;

        // Use global resolver in state.rs (hardcoded map for MVP)
        let resolver = crate::state::resolve_token(token).ok_or((StatusCode::UNAUTHORIZED, "unknown token".into()))?;
        Ok(resolver)
    }
}
