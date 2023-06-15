#![windows_subsystem = "windows"]

use iced::widget::{button, column, text};
use iced::{Alignment, Element, Sandbox, Settings};

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

impl Sandbox for App {
	type Message = Message;

	fn new() -> Self {
		Self {
			url: url::Url::parse("https://s3-us-east-2.amazonaws.com/le-mod-bucket/manifest.json").unwrap(),
			mc_path: util::minecraft_path()
		}
	}

	fn title(&self) -> String {
		String::from("green updater")
	}

	fn update(&mut self, message: Message) {
		match message {
			Message::SetMCPath(path) => self.mc_path = path,
			Message::Upgrade => todo!("upgrade")
		}
	}

	fn view(&self) -> Element<Message> {
		column![
			text("green updater").size(50),
			text("(licensed under GPL-3.0 or later)"),
			text(format!("{:?}", self.mc_path)),
			button("upgrade").on_press(Message::Upgrade),
		].align_items(Alignment::Center).into()
	}
}

fn main() {
	App::run(Settings::default()).unwrap();
}
