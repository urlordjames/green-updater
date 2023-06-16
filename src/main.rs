#![windows_subsystem = "windows"]

use iced::widget::{button, column, container, text};
use iced::{Alignment, Application, Command, Length, Element, Settings, Theme};

use std::path::PathBuf;

use green_lib::util;

struct App {
	url: url::Url,
	mc_path: PathBuf,
	can_upgrade: bool
}

#[derive(Debug, Clone)]
enum Message {
	SetMCPath(PathBuf),
	Upgrade,
	UpgradeFinished
}

impl Application for App {
	type Message = Message;
	type Theme = Theme;
	type Executor = iced::executor::Default;
	type Flags = ();

	fn new(_flags: ()) -> (Self, Command<Message>) {
		(Self {
			url: url::Url::parse("https://s3-us-east-2.amazonaws.com/le-mod-bucket/manifest.json").unwrap(),
			mc_path: util::minecraft_path(),
			can_upgrade: true
		}, Command::none())
	}

	fn title(&self) -> String {
		String::from("green updater")
	}

	fn update(&mut self, message: Message) -> Command<Message> {
		match message {
			Message::SetMCPath(path) => {
				self.mc_path = path;
				Command::none()
			},
			Message::Upgrade => {
				self.can_upgrade = false;

				let url = self.url.clone();
				let mc_path = self.mc_path.clone();

				Command::perform(async move {
					let remote_dir = green_lib::Directory::from_url(url).await.unwrap();
					remote_dir.upgrade_game_folder(&mc_path, None).await;
				}, |_| Message::UpgradeFinished)
			},
			Message::UpgradeFinished => {
				self.can_upgrade = true;
				Command::none()
			}
		}
	}

	fn view(&self) -> Element<Message> {
		let mut upgrade_button = button("upgrade");

		if self.can_upgrade {
			upgrade_button = upgrade_button.on_press(Message::Upgrade);
		}

		let content = column![
			text("green updater").size(50),
			text("(licensed under GPL-3.0 or later)"),
			text(format!("{:?}", self.mc_path)),
			upgrade_button,
			button("set to test path").on_press(Message::SetMCPath(PathBuf::from("/tmp/test")))
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
