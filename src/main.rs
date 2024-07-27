#![windows_subsystem = "windows"]

use iced::widget::{button, Column, container, text, progress_bar, mouse_area, pick_list};
use iced::{Alignment, Application, Command, Length, Subscription, Element, Settings, Theme};
use iced::futures::SinkExt;
use iced::subscription::channel;

use tokio::sync::mpsc;

use std::path::PathBuf;
use std::sync::Arc;

use green_lib::UpgradeStatus;
use green_lib::packs::PacksListManifest;

mod notify;
use notify::notify_upgrade_done;

const PACKS_URL: &str = "https://le-mod-bucket.s3.us-east-2.amazonaws.com/packs.json";

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
	packs: Option<Arc<PacksListManifest>>,
	selected_pack: Option<Arc<String>>,
	mc_path: Option<Arc<PathBuf>>,
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
	UpgradeFinished,
	PacksFetched(PacksListManifest),
	SelectPack(String)
}

macro_rules! pack_selection {
	($packs_list:ident, $selected_pack:ident) => {
		match $selected_pack {
			Some(pack) => $packs_list.packs.get(pack.as_ref()).expect("selected_pack should be valid").to_directory().await.unwrap(),
			None => $packs_list.get_featured_pack_metadata().unwrap().to_directory().await.unwrap()
		}
	};
}

impl Application for App {
	type Message = Message;
	type Theme = Theme;
	type Executor = iced::executor::Default;
	type Flags = ();

	fn new(_flags: ()) -> (Self, Command<Message>) {
		(Self {
			packs: None,
			selected_pack: None,
			#[cfg(not(feature = "flatpak"))]
			mc_path: Some(Arc::new(green_lib::util::minecraft_path())),
			#[cfg(feature = "flatpak")]
			mc_path: None,
			upgrade_state: UpgradeState::Idle,
			worker: None,
			can_select_path: true
		}, Command::none())
	}

	fn title(&self) -> String {
		match &self.upgrade_state {
			UpgradeState::FetchingDirectory => String::from("green updater: fetching..."),
			UpgradeState::Upgrading(status) if status.total == 0.0 => String::from("green updater: processing..."),
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

				let path_not_set = self.mc_path.is_none();
				Command::perform(async move {
					let mut dialog = rfd::AsyncFileDialog::new();
					if path_not_set {
						dialog = dialog.set_directory(green_lib::util::minecraft_path());
					}
					dialog.pick_folder().await.map(PathBuf::from)
				}, Message::SetMCPath)
			},
			Message::SetMCPath(path) => {
				if let Some(path) = path {
					self.mc_path = Some(Arc::new(path));
				}
				self.can_select_path = true;
				Command::none()
			},
			Message::Upgrade => {
				self.upgrade_state = UpgradeState::FetchingDirectory;
				let packs_list = self.packs.clone();
				let selected_pack = self.selected_pack.clone();

				Command::perform(async move {
					match packs_list {
						Some(packs_list) => pack_selection!(packs_list, selected_pack),
						None => {
							let packs_list = PacksListManifest::from_url(PACKS_URL).await.unwrap();
							pack_selection!(packs_list, selected_pack)
						}
					}
				}, Message::DirectoryFetched)
			},
			Message::DirectoryFetched(directory) => {
				let worker = self.worker.as_ref().unwrap().clone();
				let mc_path = self.mc_path.clone().expect("button should only be clickable if mc_path is not None");

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
			},
			Message::PacksFetched(packs) => {
				self.packs = Some(Arc::new(packs));
				Command::none()
			},
			Message::SelectPack(pack_id) => {
				self.selected_pack = Some(Arc::new(pack_id));
				Command::none()
			}
		}
	}

	fn view(&self) -> Element<Message> {
		let idle = self.can_select_path && matches!(self.upgrade_state, UpgradeState::Idle);

		let mut upgrade_button = button("upgrade");
		if idle && self.mc_path.is_some() {
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
		];

		if let Some(packs_list) = &self.packs {
			let pack_ids: Vec<String> = packs_list.packs.keys().cloned().collect();
			content.push(
				pick_list(pack_ids, packs_list.featured_pack.clone(), Message::SelectPack).into()
			);
		}

		if let Some(mc_path) = &self.mc_path {
			content.push(text(mc_path.display()).into());
		}

		content.extend([
			select_button.into(),
			upgrade_button.into()
		]);

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

			{
				let mut output = output.clone();
				tokio::spawn(async move {
					if let Some(packs_list) = PacksListManifest::from_url(PACKS_URL).await {
						output.send(Message::PacksFetched(packs_list)).await.unwrap();
					}
				});
			}

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
