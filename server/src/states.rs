use {
	axum::{
		async_trait,
		body::Bytes,
		extract::{FromRequest, Request},
		http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
		response::{IntoResponse, Response},
	},
	borsh::{BorshDeserialize, BorshSerialize},
	project_toot_sdk::IWebSocketResponse,
	std::{collections::HashMap, sync::Arc},
	tokio::sync::{mpsc::UnboundedSender, RwLock},
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub(crate) enum BorshRejection {
	IncorrectContentType,
	InvalidData,
	CouldntCatchBytes,
}

impl IntoResponse for BorshRejection {
	fn into_response(self) -> Response {
		match self {
			Self::IncorrectContentType => {
				(StatusCode::BAD_REQUEST, "Incorrect 'Content-Type' header")
			},
			Self::InvalidData => (StatusCode::BAD_REQUEST, "Invalid data provided"),
			Self::CouldntCatchBytes => (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't catch bytes"),
		}
		.into_response()
	}
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub(crate) struct Borsh<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for Borsh<T>
where
	T: BorshSerialize + BorshDeserialize,
	S: Send + Sync,
{
	type Rejection = BorshRejection;

	async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
		if !borsh_content_type(req.headers()) {
			return Err(BorshRejection::IncorrectContentType)
		}

		Ok(Self(
			borsh::from_slice::<T>(
				&Bytes::from_request(req, state)
					.await
					.or(Err(BorshRejection::CouldntCatchBytes))?,
			)
			.or(Err(BorshRejection::InvalidData))?,
		))
	}
}

impl<T: BorshSerialize + BorshDeserialize> IntoResponse for Borsh<T> {
	fn into_response(self) -> Response {
		if let Ok(data) = borsh::to_vec(&self.0) {
			([(CONTENT_TYPE, "application/borsh")], data).into_response()
		} else {
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				"Failed to serialize response",
			)
				.into_response()
		}
	}
}

fn borsh_content_type(headers: &HeaderMap) -> bool {
	if let Some(content_type) = headers.get(CONTENT_TYPE) {
		if let Ok("application/borsh") = content_type.to_str() {
			true
		} else {
			false
		}
	} else {
		false
	}
}

pub(crate) type DBManager = bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>;
pub(crate) type DBPool = bb8::Pool<DBManager>;
pub(crate) type WebSockets = HashMap<[u8; 32], UnboundedSender<IWebSocketResponse>>;

#[derive(Clone, Debug)]
pub(crate) struct AppState {
	pub pool: DBPool,
	pub websockets: Arc<RwLock<WebSockets>>,
}
