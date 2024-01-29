mod client;
mod gui;
mod messages;
mod states;
mod subscriptions;
mod tasks;
mod theme;
mod utils;
mod views;
use iced::{window::Position, Application};

fn main() -> iced::Result {
	gui::ProjectToot::run(iced::Settings {
		window: iced::window::Settings {
			position: Position::Centered,
			min_size: Some((960, 540)),
			..Default::default()
		},
		..Default::default()
	})
}
