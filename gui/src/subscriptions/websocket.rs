use {
	crate::messages::{ProjectTootMessage, WebSocketMessage},
	futures::{
		channel::mpsc::Sender,
		sink::SinkExt,
		stream::{SplitSink, SplitStream, StreamExt, TryStreamExt},
	},
	iced::subscription::{self, Subscription},
	project_toot_sdk::{IWebSocketRequest, IWebSocketResponse},
	tokio::{
		net::TcpStream,
		sync::mpsc::{self, Receiver},
	},
	tokio_tungstenite::{
		tungstenite::{self, Message},
		MaybeTlsStream,
		WebSocketStream,
	},
	url::Url,
};

async fn requests_handler(
	mut ws_tx: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
	mut req_rx: Receiver<IWebSocketRequest>,
) -> (
	SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
	Receiver<IWebSocketRequest>,
) {
	while let Some(req) = req_rx.recv().await {
		if ws_tx
			.send(Message::Binary(borsh::to_vec(&req).unwrap()))
			.await
			.is_err()
		{
			break;
		}
	}

	(ws_tx, req_rx)
}

async fn responses_handler(
	mut ws_rx: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
	mut output: Sender<ProjectTootMessage>,
) -> (
	SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
	tungstenite::Error,
) {
	let error = loop {
		match ws_rx.try_next().await {
			Ok(Some(Message::Binary(bytes))) => {
				if let Ok(resp) = borsh::from_slice::<IWebSocketResponse>(&bytes) {
					match resp {
						IWebSocketResponse::Authorized => {
							let _ = output.send(WebSocketMessage::Authorized.into()).await;
						},
						IWebSocketResponse::NewMessage(msg) => {
							let _ = output.send(WebSocketMessage::NewMessage(msg).into()).await;
						},
					}
				}
			},
			Ok(_) => (),
			Err(error) => break error,
		}
	};

	(ws_rx, error)
}

pub fn subscribe(baseurl: Url) -> Subscription<ProjectTootMessage> {
	struct WebSocketWorker;

	subscription::channel(
		std::any::TypeId::of::<WebSocketWorker>(),
		128,
		|mut output| async move {
			loop {
				let mut wsurl = baseurl.clone();
				let _ = wsurl.set_scheme("ws");
				match tokio_tungstenite::connect_async(wsurl.join("/ws").unwrap()).await {
					Ok((websocket, _)) => {
						let (req_tx, req_rx) = mpsc::channel::<IWebSocketRequest>(64);
						let (ws_tx, ws_rx) = websocket.split();

						let resp_handle = tokio::spawn(responses_handler(ws_rx, output.clone()));
						let req_handle = tokio::spawn(requests_handler(ws_tx, req_rx));

						let _ = output
							.send(WebSocketMessage::Connected(req_tx).into())
							.await;

						if let Ok((_, error)) = resp_handle.await {
							match error {
								tungstenite::Error::ConnectionClosed |
								tungstenite::Error::AlreadyClosed => (),
								ref other => eprintln!("{:#?}", other),
							}
						}
						if !req_handle.is_finished() {
							req_handle.abort();
						}

						let _ = output.send(WebSocketMessage::Disconnected.into()).await;
					},
					Err(error) => eprintln!("{:#?}", error),
				}
			}
		},
	)
}
