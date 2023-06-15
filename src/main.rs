#![windows_subsystem = "windows"]

use iced::widget::{button, column, container, text};
use iced::{Alignment, Application, Command, Length, Element, Settings, Theme};

use std::path::PathBuf;

use green_lib::util;

struct App {
	url: url::Url,
	mc_path: PathBuf
}

#[derive(Debug, Clone)]
enum Message {
	SetMCPath(PathBuf),
	Upgrade
}

impl Application for App {
	type Message = Message;
	type Theme = Theme;
	type Executor = iced::executor::Default;
	type Flags = ();

	fn new(_flags: ()) -> (Self, Command<Message>) {
		(Self {
			url: url::Url::parse("https://s3-us-east-2.amazonaws.com/le-mod-bucket/manifest.json").unwrap(),
			mc_path: util::minecraft_path()
		}, Command::none())
	}

	fn title(&self) -> String {
		String::from("green updater")
	}

	fn update(&mut self, message: Message) -> Command<Message> {
		match message {
			Message::SetMCPath(path) => self.mc_path = path,
			Message::Upgrade => todo!("upgrade")
		};

		Command::none()
	}

	fn view(&self) -> Element<Message> {
		let content = column![
			text("green updater").size(50),
			text("(licensed under GPL-3.0 or later)"),
			text(format!("{:?}", self.mc_path)),
			button("upgrade").on_press(Message::Upgrade),
		].align_items(Alignment::Center);

		container(content)
			.width(Length::Fill)
			.height(Length::Fill)
			.center_x()
			.center_y()
			.into()
	}
}

fn main() {
	App::run(Settings::default()).unwrap();
}
