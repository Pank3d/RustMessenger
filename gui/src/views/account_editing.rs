use {
	crate::{
		messages::AccountEditingMessage,
		states::Account,
		theme::{ButtonStyle, TextInputStyle},
	},
	iced::{
		alignment::{Alignment, Horizontal, Vertical},
		widget::{button, column, container, row, text, text_input},
		Element,
		Length,
		Renderer,
	},
};

pub(crate) fn view<'a, Message, Theme>(
	account: &Account,
	input: &'a str,
) -> Element<'a, Message, Renderer<Theme>>
where
	Message: 'a + Clone + From<AccountEditingMessage>,
	Theme: 'a
		+ Default
		+ button::StyleSheet<Style = ButtonStyle>
		+ container::StyleSheet
		+ text::StyleSheet
		+ text_input::StyleSheet<Style = TextInputStyle>,
{
	container(
		column![
			button(text(bs58::encode(account.address).into_string()))
				.on_press(AccountEditingMessage::Copy.into())
				.style(ButtonStyle::Clipboard),
			text_input("Account name", input)
				.size(24)
				.width(Length::Fixed(512.0))
				.padding(8)
				.on_input(|new_value| AccountEditingMessage::UpdateName(new_value).into())
				.on_paste(|new_value| AccountEditingMessage::UpdateName(new_value).into())
				.on_submit(AccountEditingMessage::End(true).into())
				.style(TextInputStyle::Dialog),
			row![
				button(text("Cancel").size(24))
					.padding([4, 8])
					.on_press(AccountEditingMessage::End(false).into()),
				button(text("Confirm").size(24))
					.padding([4, 8])
					.on_press(AccountEditingMessage::End(true).into()),
			]
			.spacing(4),
		]
		.spacing(12)
		.align_items(Alignment::Center),
	)
	.width(Length::Fill)
	.height(Length::Fill)
	.align_x(Horizontal::Center)
	.align_y(Vertical::Center)
	.into()
}
