use {
	crate::{
		messages::ChatMessage,
		states::{self, Account, Contact},
		theme::{ButtonStyle, ContainerStyle},
		utils::small_public_key,
	},
	chrono::TimeZone,
	iced::{
		alignment::{Alignment, Horizontal, Vertical},
		widget::{button, column, container, row, scrollable, text, text_input, tooltip, Column},
		Element,
		Length,
		Renderer,
	},
};

pub(crate) fn view<'a, Message, Theme>(
	accounts: &'a Vec<Account>,
	contacts: &'a Vec<Contact>,
	messages: &'a Vec<states::Message>,
	account_index: usize,
	contact_index: Option<usize>,
	message_text: &'a str,
) -> Element<'a, Message, Renderer<Theme>>
where
	Message: 'a + Clone + From<ChatMessage>,
	Theme: 'a
		+ Default
		+ button::StyleSheet<Style = ButtonStyle>
		+ container::StyleSheet<Style = ContainerStyle>
		+ scrollable::StyleSheet
		+ text::StyleSheet
		+ text_input::StyleSheet,
{
	let account = &accounts[account_index];

	row![
		container(column![
			container(
				row![
					row![
						text(account.name.clone()).size(22),
						text(format!("({})", small_public_key(&account.address))).size(12),
					]
					.spacing(4)
					.width(Length::Fill),
					button(
						text("+")
							.size(22)
							.horizontal_alignment(Horizontal::Center)
							.vertical_alignment(Vertical::Center),
					)
					.width(40)
					.height(40)
					.on_press(ChatMessage::CreateContact.into())
					.style(ButtonStyle::Contact),
				]
				.spacing(4)
				.align_items(Alignment::Center),
			)
			.padding([4, 6, 4, 8])
			.width(Length::Fill)
			.height(48),
			scrollable(contacts.iter().enumerate().fold(
				Column::new(),
				|items, (index, contact)| {
					let maybe =
						if contact_index.is_none() || contact_index.as_ref().unwrap() != &index {
							Some(ChatMessage::ChooseContact(index).into())
						} else {
							None
						};

					items.push(
						button(
							row![
								text(contact.name.clone()).size(22),
								text(format!("({})", small_public_key(&contact.address))).size(12),
							]
							.spacing(4),
						)
						.padding([4, 6, 4, 8])
						.width(Length::Fill)
						.on_press_maybe(maybe)
						.style(ButtonStyle::Contact),
					)
				}
			)),
		])
		.width(384)
		.height(Length::Fill)
		.style(ContainerStyle::Contacts),
		if let Some(index) = contact_index {
			let contact = &contacts[index];

			column![
				container(
					row![
						row![
							text(contact.name.clone()).size(24),
							text(format!("({})", small_public_key(&contact.address))).size(12)
						]
						.spacing(2)
						.width(Length::Fill),
						button(text("Delete").size(24))
							.on_press(ChatMessage::DeleteContact(index).into())
							.style(ButtonStyle::Contact)
					]
					.align_items(Alignment::Center),
				)
				.padding([4, 6, 4, 8])
				.width(Length::Fill)
				.height(48)
				.align_y(Vertical::Center)
				.style(ContainerStyle::Contact),
				scrollable(
					messages
						.iter()
						.filter(|x| {
							x.sender == contact.address && x.receiver == account.address ||
								x.sender == account.address && x.receiver == contact.address
						})
						.fold(Column::new(), |items, message| {
							items.push(
								container(
									container(
										row![
											text(message.content.clone()).size(18),
											tooltip(
												text(
													chrono::Utc
														.timestamp_micros(message.timestamp)
														.single()
														.unwrap()
														.naive_utc()
														.format("%H:%M")
												)
												.size(12),
												chrono::Utc
													.timestamp_micros(message.timestamp)
													.single()
													.unwrap()
													.naive_utc()
													.format("%B %d %Y %H:%M:%S"),
												tooltip::Position::Top,
											)
											.size(12)
											.gap(4)
											.padding(4)
											.style(ContainerStyle::MessageSent),
										]
										.align_items(Alignment::End)
										.spacing(4),
									)
									.padding(8)
									.style(ContainerStyle::Message(
										message.success,
										message.sender == account.address,
									)),
								)
								.width(Length::Fill)
								.align_x(
									if message.sender == account.address {
										Horizontal::Right
									} else {
										Horizontal::Left
									},
								),
							)
						})
						.spacing(4)
						.padding(16),
				)
				.width(Length::Fill)
				.height(Length::Fill),
				container(
					text_input("Type message...", message_text)
						.size(24)
						.padding(8)
						.on_input(|new_value| ChatMessage::UpdateMessage(new_value).into())
						.on_paste(|new_value| ChatMessage::UpdateMessage(new_value).into())
						.on_submit(ChatMessage::SendMessage.into())
				)
				.width(Length::Fill)
				.style(ContainerStyle::MessageInput),
			]
		} else {
			column![]
		}
		.width(Length::Fill)
		.height(Length::Fill),
	]
	.width(Length::Fill)
	.height(Length::Fill)
	.into()
}
