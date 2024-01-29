use {
	crate::states::AppState,
	axum::{
		extract::{ws, State},
		response::Response,
	},
	chrono::{TimeZone, Utc},
	futures::{SinkExt, StreamExt},
	project_toot_sdk::{
		ISigner,
		IWebSocketRequest,
		IWebSocketResponse,
		MAXIMUM_CONFIRMATION_TIMESTAMP_OFFSET,
	},
	sha2::{Digest, Sha512},
	tokio::sync::mpsc,
};

async fn handle_socket(state: AppState, socket: ws::WebSocket) {
	let (mut ws_tx, mut ws_rx) = socket.split();
	let (tx, mut rx) = mpsc::unbounded_channel::<IWebSocketResponse>();

	let resp_handle = tokio::task::spawn(async move {
		while let Some(message) = rx.recv().await {
			if let Err(err) = ws_tx
				.send(ws::Message::Binary(borsh::to_vec(&message).unwrap()))
				.await
			{
				eprintln!("{:#?}", err);
				break;
			}
		}
	});

	while let Some(Ok(msg)) = ws_rx.next().await {
		match msg {
			ws::Message::Binary(bytes) => {
				if let Ok(data) = borsh::from_slice::<IWebSocketRequest>(&bytes) {
					match data {
						IWebSocketRequest::Authorize(ISigner {
							address,
							salt,
							timestamp,
							signature,
						}) => {
							let now = Utc::now();
							let datetime =
								if let Some(ret) = Utc.timestamp_micros(timestamp).single() {
									ret
								} else {
									continue;
								};
							if datetime < (now - MAXIMUM_CONFIRMATION_TIMESTAMP_OFFSET) ||
								datetime > now
							{
								continue;
							}

							if address
								.verify_prehashed_strict(
									{
										let mut hasher: Sha512 = Digest::new();
										hasher.update(b"authorize");
										hasher.update(&salt);
										hasher.update(timestamp.to_le_bytes());
										hasher
									},
									None,
									&signature,
								)
								.is_err()
							{
								continue;
							}

							state
								.websockets
								.write()
								.await
								.entry(address.to_bytes())
								.or_insert(tx.clone());

							let _ = tx.send(IWebSocketResponse::Authorized);
						},
					}
				}
			},
			_ => (),
		}
	}

	if !resp_handle.is_finished() {
		resp_handle.abort();
	}
}

pub(crate) async fn handler(State(state): State<AppState>, ws: ws::WebSocketUpgrade) -> Response {
	ws.on_upgrade(|socket| handle_socket(state, socket))
}
