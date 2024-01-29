use {
	crate::{messages::ContactCreationMessage, theme::TextInputStyle},
	iced::{
		alignment::{Alignment, Horizontal, Vertical},
		widget::{button, column, container, row, text, text_input},
		Element,
		Length,
		Renderer,
	},
};

pub(crate) fn view<'a, Message, Theme>(
	address: &'a str,
	name: &'a str,
) -> Element<'a, Message, Renderer<Theme>>
where
	Message: 'a + Clone + From<ContactCreationMessage>,
	Theme: 'a
		+ Default
		+ button::StyleSheet
		+ container::StyleSheet
		+ text::StyleSheet
		+ text_input::StyleSheet<Style = TextInputStyle>,
{
	container(
		column![
			text_input("Address", address)
				.size(24)
				.width(Length::Fixed(512.0))
				.padding(8)
				.on_input(|new_value| ContactCreationMessage::UpdateAddress(new_value).into())
				.on_paste(|new_value| ContactCreationMessage::UpdateAddress(new_value).into())
				.style(TextInputStyle::Dialog),
			text_input("Name", name)
				.size(24)
				.width(Length::Fixed(512.0))
				.padding(8)
				.on_input(|new_value| ContactCreationMessage::UpdateName(new_value).into())
				.on_paste(|new_value| ContactCreationMessage::UpdateName(new_value).into())
				.style(TextInputStyle::Dialog),
			row![
				button(text("Cancel").size(24))
					.padding([4, 8])
					.on_press(ContactCreationMessage::End(false).into()),
				button(text("Create").size(24))
					.padding([4, 8])
					.on_press(ContactCreationMessage::End(true).into()),
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
