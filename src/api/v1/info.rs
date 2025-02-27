use crate::{
    keys::{Keys, KEYS},
    AppConfig, SharedState,
};
use axum::{
    body::StreamBody,
    response::{IntoResponse, Response},
    Extension, Json,
};
use axum_extra::response::ErasedJson;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct StatusResponse {
    lobby_size:        usize,
    num_contributions: usize,
}

impl IntoResponse for StatusResponse {
    fn into_response(self) -> Response {
        let status = StatusCode::OK;
        (status, Json(self)).into_response()
    }
}

pub async fn status(Extension(store): Extension<SharedState>) -> StatusResponse {
    let app_state = store.read().await;

    let lobby_size = app_state.lobby.len();
    let num_contributions = app_state.num_contributions;

    StatusResponse {
        lobby_size,
        num_contributions,
    }
}

pub async fn current_state(Extension(config): Extension<AppConfig>) -> impl IntoResponse {
    let f = match File::open(config.transcript_file).await {
        Ok(file) => file,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "could not open transcript file",
            ))
        }
    };
    let stream = ReaderStream::new(f);
    let body = StreamBody::new(stream);
    Ok((StatusCode::OK, body))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtInfoResponse {
    alg:         &'static str,
    rsa_pem_key: String,
}

impl IntoResponse for JwtInfoResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, ErasedJson::pretty(self)).into_response()
    }
}

// Returns the relevant JWT information
#[allow(clippy::unused_async)] // Required for axum function signature
pub async fn jwt_info() -> JwtInfoResponse {
    let rsa_public_key_pem_as_string = KEYS.get().unwrap().decode_key_to_string();

    JwtInfoResponse {
        alg:         Keys::alg_str(),
        rsa_pem_key: rsa_public_key_pem_as_string,
    }
}
