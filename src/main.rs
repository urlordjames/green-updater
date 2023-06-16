#![windows_subsystem = "windows"]

use iced::widget::{button, Column, container, text, progress_bar};
use iced::{Alignment, Application, Command, Length, Element, Settings, Theme};

use std::path::PathBuf;

use green_lib::util;

struct UpgradingStatus {
	total: f32,
	value: f32
}

enum UpgradeStatus {
	Upgrading(UpgradingStatus),
	Idle
}

struct App {
	url: url::Url,
	mc_path: PathBuf,
	upgrade_status: UpgradeStatus
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
			upgrade_status: UpgradeStatus::Idle
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
				self.upgrade_status = UpgradeStatus::Upgrading(UpgradingStatus {
					total: 0.0,
					value: 0.0
				});

				let url = self.url.clone();
				let mc_path = self.mc_path.clone();

				Command::perform(async move {
					let remote_dir = green_lib::Directory::from_url(url).await.unwrap();
					remote_dir.upgrade_game_folder(&mc_path, None).await;
				}, |_| Message::UpgradeFinished)
			},
			Message::UpgradeFinished => {
				self.upgrade_status = UpgradeStatus::Idle;
				Command::none()
			}
		}
	}

	fn view(&self) -> Element<Message> {
		let mut upgrade_button = button("upgrade");

		if matches!(self.upgrade_status, UpgradeStatus::Idle) {
			upgrade_button = upgrade_button.on_press(Message::Upgrade);
		}

		let mut content = vec![
			text("green updater").size(50).into(),
			text("(licensed under GPL-3.0 or later)").into(),
			text(format!("{:?}", self.mc_path)).into(),
			upgrade_button.into(),
			button("set to test path").on_press(Message::SetMCPath(PathBuf::from("/tmp/test"))).into()
		];

		if let UpgradeStatus::Upgrading(status) = &self.upgrade_status {
			content.push(progress_bar(0.0..=status.total, status.value).into());
		}

		let content = Column::with_children(content).align_items(Alignment::Center);

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
