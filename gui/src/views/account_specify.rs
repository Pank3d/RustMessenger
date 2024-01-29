use {
	crate::{
		messages::AccountSpecifyMessage,
		states::Account,
		theme::{ButtonStyle, TextStyle},
		utils::small_public_key,
	},
	iced::{
		alignment::{Alignment, Horizontal, Vertical},
		widget::{button, column, horizontal_space, row, scrollable, text, Column},
		Element,
		Length,
		Renderer,
	},
};

pub(crate) fn view<'a, Message, Theme>(
	accounts: &'a Vec<Account>,
) -> Element<'a, Message, Renderer<Theme>>
where
	Message: 'a + Clone + From<AccountSpecifyMessage>,
	Theme: 'a
		+ Default
		+ button::StyleSheet<Style = ButtonStyle>
		+ scrollable::StyleSheet
		+ text::StyleSheet<Style = TextStyle>,
{
	column![
		text("Choose an account!").size(40),
		row![
			horizontal_space(Length::FillPortion(1)),
			if accounts.len() != 0 {
				scrollable(
					accounts
						.iter()
						.enumerate()
						.fold(Column::new(), |items, (index, account)| {
							items.push(
								row![
									button(
										row![
											text(account.name.clone()).size(22),
											text(format_args!(
												"({})",
												small_public_key(&account.address),
											))
											.size(12),
										]
										.spacing(4),
									)
									.padding([4, 8])
									.width(Length::Fill)
									.on_press(AccountSpecifyMessage::Choose(index).into())
									.style(ButtonStyle::Account),
									button(text("Edit").size(22))
										.padding([4, 8])
										.on_press(AccountSpecifyMessage::Edit(index).into())
										.style(ButtonStyle::Account),
									button(text("Delete").size(22))
										.padding([4, 8])
										.on_press(AccountSpecifyMessage::Delete(index).into())
										.style(ButtonStyle::Account),
								]
								.spacing(4),
							)
						})
						.spacing(4),
				)
			} else {
				scrollable(
					text("There is no accounts exist.")
						.size(44)
						.horizontal_alignment(Horizontal::Center)
						.vertical_alignment(Vertical::Center)
						.style(TextStyle::Disabled),
				)
			}
			.width(Length::FillPortion(4)),
			horizontal_space(Length::FillPortion(1)),
		]
		.height(Length::Fill),
		button(text("Create").size(28))
			.padding([4, 8])
			.on_press(AccountSpecifyMessage::Create.into()),
	]
	.spacing(12)
	.padding(8)
	.width(Length::Fill)
	.height(Length::Fill)
	.align_items(Alignment::Center)
	.into()
}
