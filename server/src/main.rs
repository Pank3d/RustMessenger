mod cdn;
mod rpc;
mod states;
mod websocket;

use {
	crate::states::{AppState, DBManager, DBPool},
	axum::{
		extract::DefaultBodyLimit,
		routing::{get, post},
		Router,
	},
	std::{collections::HashMap, sync::Arc},
	tokio::{net, sync::RwLock},
};

#[tokio::main]
async fn main() {
	let manager = DBManager::new(
		tokio_postgres::Config::new()
			.host("localhost")
			.user("project-toot")
			.password("toot-tcejorp")
			.clone(),
		tokio_postgres::NoTls,
	);
	let pool = DBPool::builder().max_size(32).build(manager).await.unwrap();
	pool.get()
		.await
		.unwrap()
		.execute(include_str!("../sql/tables.sql"), &[])
		.await
		.unwrap();

	let app = Router::new()
		.route(
			"/cdn/:file",
			get(cdn::handler).layer(DefaultBodyLimit::max(0)),
		)
		.route(
			"/rpc",
			post(rpc::handler).layer(DefaultBodyLimit::max(1024usize.pow(2) * 6)),
		)
		.route("/ws", get(websocket::handler))
		.with_state(AppState {
			pool,
			websockets: Arc::new(RwLock::new(HashMap::new())),
		});
	let listener = net::TcpListener::bind("localhost:8080").await.unwrap();

	axum::serve(listener, app).await.unwrap();
}
