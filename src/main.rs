#![windows_subsystem = "windows"]

use iced::widget::{button, Column, container, text, progress_bar};
use iced::{Alignment, Application, Command, Length, Subscription, Element, Settings, Theme};
use iced::futures::SinkExt;
use iced::subscription::channel;

use tokio::sync::mpsc;

use std::path::PathBuf;
use std::sync::Arc;

use green_lib::UpgradeStatus;
use green_lib::util;

struct UpgradingStatus {
	total: f32,
	value: f32
}

#[derive(Debug)]
struct UpgradeInfo {
	directory: Arc<green_lib::Directory>,
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
	worker: Option<Arc<mpsc::Sender<UpgradeInfo>>>
}

#[derive(Debug, Clone)]
enum Message {
	WorkerReady(Arc<mpsc::Sender<UpgradeInfo>>),
	SetMCPath(PathBuf),
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
			url: url::Url::parse("https://s3-us-east-2.amazonaws.com/le-mod-bucket/manifest.json").unwrap(),
			mc_path: Arc::new(util::minecraft_path()),
			upgrade_state: UpgradeState::Idle,
			worker: None
		}, Command::none())
	}

	fn title(&self) -> String {
		String::from("green updater")
	}

	fn update(&mut self, message: Message) -> Command<Message> {
		match message {
			Message::WorkerReady(worker) => {
				self.worker = Some(worker);
				Command::none()
			},
			Message::SetMCPath(path) => {
				self.mc_path = Arc::new(path);
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
				let directory = Arc::new(directory);
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
		let mut upgrade_button = button("upgrade");

		if matches!(self.upgrade_state, UpgradeState::Idle) {
			upgrade_button = upgrade_button.on_press(Message::Upgrade);
		}

		let mut content = vec![
			text("green updater").size(50).into(),
			text("(licensed under GPL-3.0 or later)").into(),
			text(format!("{:?}", self.mc_path)).into(),
			upgrade_button.into(),
			button("set to test path").on_press(Message::SetMCPath(PathBuf::from("/tmp/test"))).into()
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
			output.send(Message::WorkerReady(Arc::new(tx))).await.unwrap();

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
				let _ = notify_rust::Notification::new()
					.summary("green updater finished upgrade")
					.show();
			}

			unreachable!()
		})
	}
}

fn main() {
	App::run(Settings::default()).unwrap();
}
