use {
	crate::{
		client,
		messages::{
			AccountEditingMessage,
			AccountSpecifyMessage,
			ChatMessage,
			ContactCreationMessage,
			ProjectTootMessage,
			RusqliteMessage,
			WebSocketMessage,
		},
		states::{Account, Contact, Data, Message},
		subscriptions,
		tasks,
		theme::Theme,
		utils,
		views,
	},
	ed25519_dalek::{SigningKey, VerifyingKey},
	iced::{clipboard, Application, Command, Element, Renderer, Subscription},
	project_toot_sdk::{IMessage, ISendMessagePayload, ISigner, IWebSocketRequest, Salt},
	reqwest::Url,
	sha2::{Digest, Sha512},
	std::path::PathBuf,
	tokio::sync::mpsc::Sender,
	tokio_rusqlite::Connection,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Default)]
enum Scene {
	#[default]
	AccountSpecify,
	AccountEditing,
	ContactCreation,
	Chat,
}

pub(crate) struct ProjectToot {
	basedir: PathBuf,
	client: client::ProjectTootClient,
	database: Option<Connection>,
	sender: Option<Sender<IWebSocketRequest>>,
	data: Data,
	scene: Scene,
	account_index: Option<usize>,
	contact_index: Option<usize>,
	inputs: (String, String),
}

impl ProjectToot {
	fn account_specify_create(&mut self) -> Command<<Self as Application>::Message> {
		let secret = SigningKey::from_bytes(&rand::random());
		let address = secret.verifying_key();
		let account = Account {
			secret,
			address,
			name: format!("Account {}", utils::small_public_key(&address)),
		};

		self.data.accounts.push(account.clone());

		if let Some(database) = self.database.clone() {
			Command::perform(tasks::insert_account::run(database, account), |res| {
				if let Err(err) = res {
					eprintln!("{:#?}", err);
				}

				ProjectTootMessage::None
			})
		} else {
			Command::none()
		}
	}

	fn account_specify_edit(&mut self, index: usize) -> Command<<Self as Application>::Message> {
		self.account_index = Some(index);
		self.inputs.0 = self.data.accounts[index].name.clone();
		self.scene = Scene::AccountEditing;

		Command::none()
	}

	fn account_specify_delete(&mut self, index: usize) -> Command<<Self as Application>::Message> {
		let account = self.data.accounts.remove(index);

		if let Some(database) = self.database.clone() {
			Command::perform(
				tasks::delete_account::run(database, account.address),
				|res| {
					if let Err(err) = res {
						eprintln!("{:#?}", err);
					}

					ProjectTootMessage::None
				},
			)
		} else {
			Command::none()
		}
	}

	fn account_specify_choose(&mut self, index: usize) -> Command<<Self as Application>::Message> {
		self.account_index = Some(index);
		self.scene = Scene::Chat;

		if let Some(database) = self.database.clone() {
			Command::batch([
				Command::perform(
					tasks::load_messages::run(
						self.basedir.clone(),
						self.client.clone(),
						database,
						self.data.accounts[index].clone(),
					),
					|res| {
						if let Err(err) = res {
							eprintln!("{:#?}", err);
						}

						ProjectTootMessage::None
					},
				),
				if let Some(sender) = self.sender.clone() {
					let account = self.data.accounts[index].clone();
					let salt = rand::random::<Salt>();
					let timestamp = chrono::Utc::now().timestamp_micros();

					Command::perform(
						async move {
							sender
								.send(IWebSocketRequest::Authorize(ISigner {
									address: account.address,
									salt,
									timestamp,
									signature: {
										let mut hasher: Sha512 = Digest::new();
										hasher.update(b"authorize");
										hasher.update(salt);
										hasher.update(timestamp.to_le_bytes());
										account.secret.sign_prehashed(hasher, None).unwrap()
									},
								}))
								.await
						},
						|res| {
							if let Err(err) = res {
								eprintln!("{:#?}", err);
							}

							ProjectTootMessage::None
						},
					)
				} else {
					Command::none()
				},
			])
		} else {
			Command::none()
		}
	}

	fn account_editing_end(&mut self, confirmed: bool) -> Command<<Self as Application>::Message> {
		let command = if confirmed {
			if let Some(index) = self.account_index.clone() {
				let account = &mut self.data.accounts[index];
				account.name = self.inputs.0.clone();

				if let Some(database) = self.database.clone() {
					Command::perform(
						tasks::update_account::run(
							database,
							account.address.clone(),
							self.inputs.0.clone(),
						),
						|res| {
							if let Err(err) = res {
								eprintln!("{:#?}", err);
							}

							ProjectTootMessage::None
						},
					)
				} else {
					Command::none()
				}
			} else {
				Command::none()
			}
		} else {
			Command::none()
		};

		self.account_index = None;
		self.inputs.0 = String::new();
		self.scene = Scene::AccountSpecify;

		command
	}

	fn chat_create_contact(&mut self) -> Command<<Self as Application>::Message> {
		self.scene = Scene::ContactCreation;

		Command::none()
	}

	fn chat_delete_contact(&mut self, index: usize) -> Command<<Self as Application>::Message> {
		let contact = self.data.contacts.remove(index);
		self.contact_index = None;

		if let Some(database) = self.database.clone() {
			Command::perform(
				tasks::delete_contact::run(database, contact.address),
				|res| {
					if let Err(err) = res {
						eprintln!("{:#?}", err);
					}

					ProjectTootMessage::None
				},
			)
		} else {
			Command::none()
		}
	}

	fn chat_choose_contact(&mut self, index: usize) -> Command<<Self as Application>::Message> {
		self.contact_index = Some(index);

		if let Some(database) = self.database.as_ref() {
			Command::perform(
				tasks::get_messages::run(
					self.basedir.clone(),
					self.data.accounts[*self.account_index.as_ref().unwrap()].clone(),
					self.data.contacts[index].clone(),
					database.clone(),
				),
				|res| {
					if let Ok(messages) = res {
						RusqliteMessage::LoadedMessages(messages).into()
					} else {
						eprintln!("{:#?}", res.unwrap_err());
						ProjectTootMessage::None
					}
				},
			)
		} else {
			Command::none()
		}
	}

	fn chat_send_message(&mut self) -> Command<<Self as Application>::Message> {
		if self.inputs.0.is_empty() {
			return Command::none()
		}

		let account = self.data.accounts[self.account_index.unwrap()].clone();
		let contact = self.data.contacts[self.contact_index.unwrap()].clone();

		let data = utils::encrypt(
			&account,
			&contact.address,
			self.inputs.0.as_bytes().to_vec(),
		);
		let salt = rand::random::<Salt>();
		let timestamp = chrono::Utc::now().timestamp_micros();
		let mut hasher: Sha512 = Digest::new();
		hasher.update(b"send-message");
		hasher.update(salt);
		hasher.update(timestamp.to_le_bytes());

		self.inputs.0 = String::new();

		Command::perform(
			self.client.clone().send_message(
				ISigner {
					address: account.address,
					salt,
					timestamp,
					signature: account.secret.sign_prehashed(hasher.clone(), None).unwrap(),
				},
				ISendMessagePayload {
					receiver: contact.address,
					data: data.clone(),
				},
			),
			move |res| {
				if let Err(err) = res {
					eprintln!("{:#?}", err);
					ProjectTootMessage::None
				} else {
					ProjectTootMessage::NewMessage(
						IMessage {
							hash: hasher.finalize().into(),
							sender: account.address,
							receiver: contact.address,
							timestamp,
							data_hash: {
								let mut hasher: Sha512 = Digest::new();
								hasher.update(&data);
								hasher.finalize().into()
							},
						},
						data,
						res.is_ok(),
					)
				}
			},
		)
	}

	fn contact_creation_end(&mut self, confirmed: bool) -> Command<<Self as Application>::Message> {
		let command = if confirmed {
			let address = if let Ok(ret) = VerifyingKey::try_from(
				if let Ok(ret) = bs58::decode(self.inputs.0.as_str()).into_vec() {
					ret
				} else {
					return Command::none()
				}
				.as_slice(),
			) {
				ret
			} else {
				return Command::none()
			};
			let contact = Contact {
				address,
				name: self.inputs.1.clone(),
			};

			self.data.contacts.push(contact.clone());

			if let Some(database) = self.database.as_ref() {
				Command::perform(
					tasks::insert_contact::run(database.clone(), contact),
					|res| {
						if let Err(err) = res {
							eprintln!("{:#?}", err);
						}

						ProjectTootMessage::None
					},
				)
			} else {
				Command::none()
			}
		} else {
			Command::none()
		};

		self.inputs.0 = String::new();
		self.inputs.1 = String::new();
		self.scene = Scene::Chat;

		command
	}

	fn database_connected(
		&mut self,
		database: Connection,
	) -> Command<<Self as Application>::Message> {
		self.database = Some(database.clone());
		Command::batch([
			Command::perform(tasks::init_tables::run(database.clone()), |res| {
				if let Err(err) = res {
					eprintln!("{:#?}", err);
				}

				ProjectTootMessage::None
			}),
			Command::perform(tasks::get_accounts::run(database.clone()), |res| {
				if let Ok(accounts) = res {
					RusqliteMessage::LoadedAccounts(accounts).into()
				} else {
					eprintln!("{:#?}", res.unwrap_err());
					ProjectTootMessage::None
				}
			}),
			Command::perform(tasks::get_contacts::run(database.clone()), |res| {
				if let Ok(contacts) = res {
					RusqliteMessage::LoadedContacts(contacts).into()
				} else {
					eprintln!("{:#?}", res.unwrap_err());
					ProjectTootMessage::None
				}
			}),
		])
	}

	fn websocket_new_message(
		&mut self,
		message: IMessage,
	) -> Command<<Self as Application>::Message> {
		Command::perform(
			tasks::download::run(
				self.client.http(),
				self.client
					.baseurl()
					.join("cdn/")
					.unwrap()
					.join((hex::encode(message.data_hash) + ".dat").as_str())
					.unwrap(),
			),
			move |res| {
				if let Ok(data) = res {
					ProjectTootMessage::NewMessage(message, data, true)
				} else {
					ProjectTootMessage::None
				}
			},
		)
	}

	fn new_message(
		&mut self,
		message: IMessage,
		data: Vec<u8>,
		success: bool,
	) -> Command<<Self as Application>::Message> {
		let content = if let Some(account) = self
			.data
			.accounts
			.iter()
			.find(|x| x.address == message.sender || x.address == message.receiver)
		{
			utils::decrypt(
				account,
				&if account.address == message.sender {
					message.receiver
				} else {
					message.sender
				},
				&data,
			)
		} else {
			String::from("No account to decrypt this message")
		};

		let msg = Message {
			hash: message.hash,
			sender: message.sender,
			receiver: message.receiver,
			data_hash: message.data_hash,
			timestamp: message.timestamp,
			success,
			content,
		};

		self.data.messages.push(msg.clone());

		Command::batch([
			if let Some(database) = self.database.as_ref() {
				Command::perform(tasks::insert_message::run(database.clone(), msg), |res| {
					if let Err(err) = res {
						eprintln!("{:#?}", err);
					}

					ProjectTootMessage::None
				})
			} else {
				Command::none()
			},
			Command::perform(
				tasks::write_data::run(self.basedir.clone(), message.data_hash, data),
				|res| {
					if let Err(err) = res {
						eprintln!("{:#?}", err);
					}

					ProjectTootMessage::None
				},
			),
		])
	}
}

impl Application for ProjectToot {
	type Executor = iced::executor::Default;
	type Message = ProjectTootMessage;
	type Theme = Theme;
	type Flags = ();

	fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
		let basedir = home::home_dir()
			.expect("Failed to get home dir")
			.join(".local/share/ptd/");
		std::fs::create_dir_all(basedir.clone()).expect("Failed to create program data dir");

		(
			Self {
				basedir: basedir.clone(),
				client: client::ProjectTootClient::new(
					Url::parse("http://82.97.242.232:8080/").unwrap(),
				),
				database: None,
				sender: None,
				data: Data::default(),
				scene: Scene::AccountSpecify,
				account_index: None,
				contact_index: None,
				inputs: (String::new(), String::new()),
			},
			Command::perform(Connection::open(basedir.join("db.sqlite3")), |res| {
				if let Ok(database) = res {
					RusqliteMessage::Connected(database).into()
				} else {
					ProjectTootMessage::None
				}
			}),
		)
	}

	fn title(&self) -> String {
		String::from("Project TOOT")
	}

	fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
		match message {
			ProjectTootMessage::None => Command::none(),
			ProjectTootMessage::AccountSpecify(inner) => match inner {
				AccountSpecifyMessage::Create => self.account_specify_create(),
				AccountSpecifyMessage::Edit(index) => self.account_specify_edit(index),
				AccountSpecifyMessage::Delete(index) => self.account_specify_delete(index),
				AccountSpecifyMessage::Choose(index) => self.account_specify_choose(index),
			},
			ProjectTootMessage::AccountEditing(inner) => match inner {
				AccountEditingMessage::UpdateName(new_value) => {
					self.inputs.0 = new_value;
					Command::none()
				},
				AccountEditingMessage::End(confirmed) => self.account_editing_end(confirmed),
				AccountEditingMessage::Copy => clipboard::write(utils::public_key(
					&self.data.accounts[*self.account_index.as_ref().unwrap()].address,
				)),
			},
			ProjectTootMessage::Chat(inner) => match inner {
				ChatMessage::CreateContact => self.chat_create_contact(),
				ChatMessage::DeleteContact(index) => self.chat_delete_contact(index),
				ChatMessage::ChooseContact(index) => self.chat_choose_contact(index),
				ChatMessage::UpdateMessage(new_value) => {
					self.inputs.0 = new_value;
					Command::none()
				},
				ChatMessage::SendMessage => self.chat_send_message(),
			},
			ProjectTootMessage::ContactCreation(inner) => match inner {
				ContactCreationMessage::UpdateAddress(new_value) => {
					self.inputs.0 = new_value;
					Command::none()
				},
				ContactCreationMessage::UpdateName(new_value) => {
					self.inputs.1 = new_value;
					Command::none()
				},
				ContactCreationMessage::End(confirmed) => self.contact_creation_end(confirmed),
			},
			ProjectTootMessage::Database(inner) => match inner {
				RusqliteMessage::Connected(database) => self.database_connected(database),
				RusqliteMessage::LoadedAccounts(accounts) => {
					self.data.accounts.extend(accounts.into_iter());
					Command::none()
				},
				RusqliteMessage::LoadedContacts(contacts) => {
					self.data.contacts.extend(contacts.into_iter());
					Command::none()
				},
				RusqliteMessage::LoadedMessages(messages) => {
					self.data.messages = messages;
					Command::none()
				},
			},
			ProjectTootMessage::WebSocket(inner) => match inner {
				WebSocketMessage::Connected(sender) => {
					self.sender = Some(sender);
					Command::none()
				},
				WebSocketMessage::Disconnected => {
					self.sender = None;
					Command::none()
				},
				WebSocketMessage::Authorized => {
					println!("Successfully authorized with websocket!");
					Command::none()
				},
				WebSocketMessage::NewMessage(msg) => self.websocket_new_message(msg),
			},
			ProjectTootMessage::NewMessage(msg, data, success) => {
				self.new_message(msg, data, success)
			},
		}
	}

	fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
		match self.scene {
			Scene::AccountSpecify => views::account_specify::view(&self.data.accounts),
			Scene::AccountEditing => views::account_editing::view(
				&self.data.accounts[self.account_index.unwrap()],
				&self.inputs.0,
			),
			Scene::ContactCreation => views::contact_creation::view(&self.inputs.0, &self.inputs.1),
			Scene::Chat => views::chat::view(
				&self.data.accounts,
				&self.data.contacts,
				&self.data.messages,
				self.account_index.unwrap(),
				self.contact_index,
				&self.inputs.0,
			),
		}
	}

	fn subscription(&self) -> Subscription<Self::Message> {
		subscriptions::websocket::subscribe(self.client.baseurl().clone())
	}
}
