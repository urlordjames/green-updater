#![windows_subsystem = "windows"]

use iced::widget::{button, Column, container, text, progress_bar, mouse_area};
use iced::{Alignment, Application, Command, Length, Subscription, Element, Settings, Theme};
use iced::futures::SinkExt;
use iced::subscription::channel;

use tokio::sync::mpsc;

use std::path::PathBuf;
use std::sync::Arc;

use green_lib::UpgradeStatus;
use green_lib::util;

mod notify;
use notify::notify_upgrade_done;

struct UpgradingStatus {
	total: f32,
	value: f32
}

#[derive(Debug)]
struct UpgradeInfo {
	directory: green_lib::Directory,
	mc_path: Arc<PathBuf>
}

enum UpgradeState {
	FetchingDirectory,
	Upgrading(UpgradingStatus),
	Idle
}

struct App {
	url: url::Url,
	mc_path: Arc<PathBuf>,
	upgrade_state: UpgradeState,
	worker: Option<mpsc::Sender<UpgradeInfo>>,
	can_select_path: bool
}

#[derive(Debug, Clone)]
enum Message {
	WorkerReady(mpsc::Sender<UpgradeInfo>),
	OpenProjectLink,
	SelectMCPath,
	SetMCPath(Option<PathBuf>),
	Upgrade,
	DirectoryFetched(green_lib::Directory),
	UpgradeInProgress,
	SetLength(f32),
	Tick,
	UpgradeFinished
}

impl Application for App {
	type Message = Message;
	type Theme = Theme;
	type Executor = iced::executor::Default;
	type Flags = ();

	fn new(_flags: ()) -> (Self, Command<Message>) {
		(Self {
			url: url::Url::parse("https://s3-us-east-2.amazonaws.com/le-mod-bucket/manifest2.json").unwrap(),
			mc_path: Arc::new(util::minecraft_path()),
			upgrade_state: UpgradeState::Idle,
			worker: None,
			can_select_path: true
		}, Command::none())
	}

	fn title(&self) -> String {
		match &self.upgrade_state {
			UpgradeState::FetchingDirectory => String::from("green updater: fetching..."),
			UpgradeState::Upgrading(status) => format!("green updater: {:.1}% downloaded", (status.value / status.total) * 100.0),
			UpgradeState::Idle => String::from("green updater")
		}
	}

	fn update(&mut self, message: Message) -> Command<Message> {
		match message {
			Message::WorkerReady(worker) => {
				self.worker = Some(worker);
				Command::none()
			},
			Message::OpenProjectLink => {
				open::that_detached("https://github.com/urlordjames/green-updater").unwrap();
				Command::none()
			},
			Message::SelectMCPath => {
				self.can_select_path = false;
				Command::perform(async move {
					let dialog = rfd::AsyncFileDialog::new();
					dialog.pick_folder().await.map(PathBuf::from)
				}, Message::SetMCPath)
			},
			Message::SetMCPath(path) => {
				if let Some(path) = path {
					self.mc_path = Arc::new(path);
				}
				self.can_select_path = true;
				Command::none()
			},
			Message::Upgrade => {
				self.upgrade_state = UpgradeState::FetchingDirectory;
				let url = self.url.clone();

				Command::perform(async move {
					green_lib::Directory::from_url(url).await.unwrap()
				}, Message::DirectoryFetched)
			},
			Message::DirectoryFetched(directory) => {
				let worker = self.worker.as_ref().unwrap().clone();
				let mc_path = self.mc_path.clone();

				Command::perform(async move {
					worker.send(UpgradeInfo {
						directory,
						mc_path
					}).await.unwrap();
				}, |_| Message::UpgradeInProgress)
			},
			Message::UpgradeInProgress => {
				self.upgrade_state = UpgradeState::Upgrading(UpgradingStatus {
					total: 0.0,
					value: 0.0
				});

				Command::none()
			},
			Message::SetLength(length) => {
				match &mut self.upgrade_state {
					UpgradeState::Upgrading(status) => status.total = length,
					_ => unreachable!()
				}

				Command::none()
			},
			Message::Tick => {
				match &mut self.upgrade_state {
					UpgradeState::Upgrading(status) => status.value += 1.0,
					_ => unreachable!()
				}

				Command::none()
			},
			Message::UpgradeFinished => {
				self.upgrade_state = UpgradeState::Idle;
				Command::none()
			}
		}
	}

	fn view(&self) -> Element<Message> {
		let idle = self.can_select_path && matches!(self.upgrade_state, UpgradeState::Idle);

		let mut upgrade_button = button("upgrade");
		if idle {
			upgrade_button = upgrade_button.on_press(Message::Upgrade);
		}

		let mut select_button = button("select Minecraft folder");
		if idle {
			select_button = select_button.on_press(Message::SelectMCPath);
		}

		let mut content = vec![
			mouse_area(
				text("green updater").size(50)
			).on_press(Message::OpenProjectLink).into(),
			text("(licensed under GPL-3.0 or later)").into(),
			text(self.mc_path.display()).into(),
			select_button.into(),
			upgrade_button.into()
		];

		if let UpgradeState::Upgrading(status) = &self.upgrade_state {
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

	fn subscription(&self) -> Subscription<Message> {
		channel(0, 128, |mut output| async move {
			let (tx, mut rx) = mpsc::channel(128);
			output.send(Message::WorkerReady(tx)).await.unwrap();

			while let Some(update) = rx.recv().await {
				let (tx, mut rx) = mpsc::channel(128);
				let handle = tokio::spawn(async move {
					update.directory.upgrade_game_folder(&update.mc_path, Some(tx)).await
				});

				while let Some(status) = rx.recv().await {
					match status {
						UpgradeStatus::Length(len) => output.send(Message::SetLength(len as f32)).await.unwrap(),
						UpgradeStatus::Tick => output.send(Message::Tick).await.unwrap()
					}
				}

				handle.await.unwrap();
				output.send(Message::UpgradeFinished).await.unwrap();
				notify_upgrade_done().await;
			}

			unreachable!()
		})
	}

	fn theme(&self) -> Theme {
		Theme::Dark
	}
}

fn main() {
	pretty_env_logger::init();
	App::run(Settings {
		window: iced::window::Settings {
			size: iced::Size::new(500.0, 400.0),
			..iced::window::Settings::default()
		},
		..Settings::default()
	}).unwrap();
}
