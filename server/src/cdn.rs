use {
	axum::{
		extract::Path,
		http::StatusCode,
		response::{IntoResponse, Response},
	},
	std::path::PathBuf,
	tokio::fs,
};

pub(crate) const CDN_PATH: &str = "./cdn/";

pub(crate) async fn handler(Path(file): Path<String>) -> Response {
	if let Ok(data) = fs::read(PathBuf::from(CDN_PATH).join(file)).await {
		data.into_response()
	} else {
		(StatusCode::NOT_FOUND, "Not found").into_response()
	}
}
