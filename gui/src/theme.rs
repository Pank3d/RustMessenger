use iced::{
	application,
	widget::{button, container, scrollable, text, text_input},
	Background,
	Color,
};

#[derive(Default)]
pub(crate) struct Theme;

impl application::StyleSheet for Theme {
	type Style = ();

	fn appearance(&self, _: &Self::Style) -> application::Appearance {
		application::Appearance {
			background_color: Color::WHITE,
			text_color: Color::BLACK,
		}
	}
}

#[derive(Clone, Copy, Default)]
pub(crate) enum ButtonStyle {
	#[default]
	Default,
	Account,
	Contact,
	Clipboard,
}

impl button::StyleSheet for Theme {
	type Style = ButtonStyle;

	fn active(&self, style: &Self::Style) -> button::Appearance {
		match style {
			ButtonStyle::Default => button::Appearance {
				background: Some(Background::Color([0.4, 0.4, 0.75].into())),
				border_radius: (8.).into(),
				..Default::default()
			},
			ButtonStyle::Account | ButtonStyle::Contact => button::Appearance {
				background: Some(Background::Color([0.0, 0.0, 0.0, 0.2].into())),
				..Default::default()
			},
			ButtonStyle::Clipboard => button::Appearance::default(),
		}
	}

	fn hovered(&self, style: &Self::Style) -> button::Appearance {
		let active = self.active(style);

		match style {
			ButtonStyle::Default => button::Appearance {
				background: Some(Background::Color([0.3, 0.3, 0.65].into())),
				..active
			},
			ButtonStyle::Account | ButtonStyle::Contact => button::Appearance {
				background: Some(Background::Color([0.0, 0.0, 0.0, 0.35].into())),
				..active
			},
			ButtonStyle::Clipboard => active,
		}
	}
}

#[derive(Clone, Copy, Default)]
pub(crate) enum ContainerStyle {
	#[default]
	Default,
	Contacts,
	Contact,
	Message(bool, bool),
	MessageInput,
	MessageSent,
}

impl container::StyleSheet for Theme {
	type Style = ContainerStyle;

	fn appearance(&self, style: &Self::Style) -> container::Appearance {
		match style {
			ContainerStyle::Default => Default::default(),
			ContainerStyle::Contacts | ContainerStyle::Contact | ContainerStyle::MessageInput => {
				container::Appearance {
					background: Some(Background::Color([0.9, 0.9, 0.9].into())),
					border_width: 1.5,
					border_color: [0.0, 0.0, 0.0, 0.25].into(),
					..Default::default()
				}
			},
			ContainerStyle::Message(success, mine) => container::Appearance {
				background: Some(Background::Color(
					if *success {
						if *mine {
							[0.9, 1.0, 0.9]
						} else {
							[0.8, 1.0, 0.8]
						}
					} else {
						if *mine {
							[1.0, 0.9, 0.9]
						} else {
							[1.0, 0.8, 0.8]
						}
					}
					.into(),
				)),
				border_radius: (8.0).into(),
				..Default::default()
			},
			ContainerStyle::MessageSent => container::Appearance {
				text_color: Some([0.75, 0.75, 0.75].into()),
				background: Some(Background::Color([0.25, 0.3, 0.3].into())),
				border_radius: 0.0.into(),
				border_width: 1.0,
				border_color: [0.5, 0.5, 0.5].into(),
			},
		}
	}
}

impl scrollable::StyleSheet for Theme {
	type Style = ();

	fn active(&self, _: &Self::Style) -> scrollable::Scrollbar {
		scrollable::Scrollbar {
			background: None,
			border_radius: 4.0.into(),
			border_width: 4.0,
			border_color: Color::TRANSPARENT,
			scroller: scrollable::Scroller {
				color: [0.4, 0.4, 0.4, 0.75].into(),
				border_radius: 8.0.into(),
				border_width: 4.0,
				border_color: Color::TRANSPARENT,
			},
		}
	}

	fn hovered(
		&self,
		style: &Self::Style,
		_is_mouse_over_scrollbar: bool,
	) -> scrollable::Scrollbar {
		scrollable::Scrollbar {
			background: Some(Background::Color([0.0, 0.0, 0.0, 0.15].into())),
			..Self::active(&self, style)
		}
	}
}

#[derive(Clone, Copy, Default)]
pub(crate) enum TextStyle {
	#[default]
	Default,
	Disabled,
}

impl text::StyleSheet for Theme {
	type Style = TextStyle;

	fn appearance(&self, style: Self::Style) -> text::Appearance {
		match style {
			TextStyle::Default => Default::default(),
			TextStyle::Disabled => text::Appearance {
				color: Some([0.0, 0.0, 0.0, 0.25].into()),
			},
		}
	}
}

#[derive(Clone, Copy, Default)]
pub(crate) enum TextInputStyle {
	#[default]
	Default,
	Dialog,
}

impl text_input::StyleSheet for Theme {
	type Style = TextInputStyle;

	fn active(&self, style: &Self::Style) -> text_input::Appearance {
		text_input::Appearance {
			background: Background::Color(match style {
				TextInputStyle::Default => Color::TRANSPARENT,
				TextInputStyle::Dialog => [0.0, 0.0, 0.0, 0.2].into(),
			}),
			border_radius: 0.0.into(),
			border_width: 2.0,
			border_color: Color::TRANSPARENT,
			icon_color: Color::BLACK,
		}
	}

	fn focused(&self, style: &Self::Style) -> text_input::Appearance {
		Self::active(&self, style)
	}

	fn placeholder_color(&self, _: &Self::Style) -> Color {
		[0.0, 0.0, 0.0, 0.4].into()
	}

	fn value_color(&self, _: &Self::Style) -> Color {
		Color::BLACK
	}

	fn disabled_color(&self, style: &Self::Style) -> Color {
		Self::placeholder_color(&self, style)
	}

	fn selection_color(&self, _: &Self::Style) -> Color {
		[0.4, 0.4, 1.0].into()
	}

	fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
		Self::active(&self, style)
	}
}
