use axum::{
    http::StatusCode, response::IntoResponse, response::Response, routing::get, Json, Router,
};
use dal::TransactionsError;
use si_data_pg::{PgError, PgPoolError};
use thiserror::Error;

use crate::server::state::AppState;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum WsError {
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
}

pub mod workspace_updates;

impl IntoResponse for WsError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(serde_json::json!({
            "error": {
                "message": error_message,
                "code": 42,
                "statusCode": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/workspace_updates",
        get(workspace_updates::workspace_updates),
    )
}
