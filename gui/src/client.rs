use {
	borsh::io,
	project_toot_sdk::{
		IDeleteMessagesPayload,
		IGetMessagesPayload,
		IMessage,
		IRPCRequest,
		IRPCResponse,
		ISendMessagePayload,
		ISigner,
	},
	reqwest::{header::CONTENT_TYPE, Client, StatusCode, Url},
};

#[derive(Debug)]
pub(crate) enum ClientError {
	IoError(io::Error),
	ReqwestError(reqwest::Error),
	FailedToMakeRequest,
	RPCError(i32),
	Unknown(String),
}

impl From<io::Error> for ClientError {
	fn from(value: io::Error) -> Self {
		Self::IoError(value)
	}
}

impl From<reqwest::Error> for ClientError {
	fn from(value: reqwest::Error) -> Self {
		Self::ReqwestError(value)
	}
}

impl From<i32> for ClientError {
	fn from(value: i32) -> Self {
		Self::RPCError(value)
	}
}

#[derive(Clone, Debug)]
pub(crate) struct ProjectTootClient {
	baseurl: Url,
	http: Client,
}

impl ProjectTootClient {
	pub fn new(baseurl: Url) -> Self {
		Self {
			baseurl,
			http: Client::new(),
		}
	}

	pub fn baseurl(&self) -> Url {
		self.baseurl.clone()
	}

	pub fn http(&self) -> Client {
		self.http.clone()
	}

	async fn request(self, data: IRPCRequest) -> Result<(u128, IRPCResponse), ClientError> {
		let resp = self
			.http
			.post(self.baseurl.join("/rpc").unwrap())
			.header(CONTENT_TYPE, "application/borsh")
			.body(borsh::to_vec(&(0u128, data)).unwrap())
			.send()
			.await
			.or(Err(ClientError::FailedToMakeRequest))?;

		match resp.status() {
			StatusCode::OK | StatusCode::BAD_REQUEST => {
				let (id, resp) =
					borsh::from_slice::<(u128, Result<IRPCResponse, i32>)>(&resp.bytes().await?)?;
				Ok((id, resp?))
			},
			status => Err(ClientError::Unknown(status.to_string())),
		}
	}

	pub async fn send_message(
		self,
		signer: ISigner,
		payload: ISendMessagePayload,
	) -> Result<(), ClientError> {
		self.request(IRPCRequest::SendMessage(signer, payload))
			.await?;
		Ok(())
	}

	pub async fn get_messages(
		self,
		signer: ISigner,
		payload: IGetMessagesPayload,
	) -> Result<Vec<IMessage>, ClientError> {
		match self
			.request(IRPCRequest::GetMessages(signer, payload))
			.await?
			.1
		{
			IRPCResponse::GetMessages(inner) => Ok(inner),
			_ => panic!("Unexpected response"),
		}
	}

	pub async fn delete_messages(
		self,
		signer: ISigner,
		payload: IDeleteMessagesPayload,
	) -> Result<Vec<bool>, ClientError> {
		match self
			.request(IRPCRequest::DeleteMessages(signer, payload))
			.await?
			.1
		{
			IRPCResponse::DeleteMessages(inner) => Ok(inner),
			_ => panic!("Unexpected response"),
		}
	}
}
