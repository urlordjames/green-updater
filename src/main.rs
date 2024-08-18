#![cfg_attr(not(feature = "env-logging"), windows_subsystem = "windows")]

use iced::widget::{button, Column, container, text, progress_bar, mouse_area, pick_list, tooltip, theme};
use iced::{Alignment, Application, Command, Length, Subscription, Element, Settings, Theme};
use iced::futures::SinkExt;
use iced::subscription::channel;

use tokio::sync::mpsc;

use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;

use green_lib::UpgradeStatus;
use green_lib::packs::{PacksListManifest, ManifestMetadata};

mod notify;
use notify::send_notification;

#[cfg(feature = "cloud-logging")]
mod cloud_logging;

#[cfg(feature = "cloud-logging")]
compile_error!("bruh AWS made me take this down");

#[cfg(all(feature = "cloud-logging", feature = "env-logging"))]
compile_error!("the features `cloud-logging` and `env-logging` are mutually exclusive");

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
	packs: Option<HashMap<String, ManifestMetadata>>,
	selected_pack: Option<Arc<PickListPack>>,
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
	FailedPackFetch,
	SelectPack(Arc<PickListPack>)
}

#[derive(Debug)]
struct PickListPack {
	id: String,
	display_name: String
}

impl std::fmt::Display for PickListPack {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{}", self.display_name)
	}
}

impl PartialEq for PickListPack {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

macro_rules! tooltip {
	($text:ident, $e:expr, $t:literal) => {
		if $e {
			$text.push('\n');
			$text.push_str($t);
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

				#[cfg(feature = "flatpak")]
				let path_not_set = self.mc_path.is_none();

				Command::perform(async move {
					let dialog = rfd::AsyncFileDialog::new();

					#[cfg(feature = "flatpak")]
					let dialog = if path_not_set {
						dialog.set_directory(green_lib::util::minecraft_path())
					} else { dialog };

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
				let packs_list = self.packs.as_ref().expect("button should only be clickable if packs_list is not None");
				let selected_pack = self.selected_pack.as_ref().expect("button should only be clickable if selected_back is not None");
				let metadata = packs_list.get(&selected_pack.id).expect("selected_pack should be valid").clone();

				Command::perform(async move {
					metadata.to_directory().await.unwrap()
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
			Message::PacksFetched(packs_manifest) => {
				if let Some(featured_pack) = packs_manifest.featured_pack {
					let pick_list_pack = PickListPack {
						display_name: packs_manifest.packs.get(&featured_pack).unwrap().display_name.clone(),
						id: featured_pack.clone()
					};
					self.selected_pack = Some(Arc::new(pick_list_pack));
				}
				self.packs = Some(packs_manifest.packs);
				Command::none()
			},
			Message::FailedPackFetch => {
				log::error!("failed to fetch packs too many times, closing application");
				iced::window::close(iced::window::Id::MAIN)
			},
			Message::SelectPack(pack) => {
				self.selected_pack = Some(pack.clone());
				Command::none()
			}
		}
	}

	fn view(&self) -> Element<Message> {
		let idle = self.can_select_path && matches!(self.upgrade_state, UpgradeState::Idle);

		let mut content = vec![
			mouse_area(
				text("green updater").size(50)
			).on_press(Message::OpenProjectLink).into(),
			text("(licensed under GPL-3.0 or later)").into(),
		];

		if idle {
			if let Some(packs_list) = &self.packs {
				let packs: Vec<Arc<PickListPack>> = packs_list.iter().map(|p| Arc::new(PickListPack {
					id: p.0.clone(),
					display_name: p.1.display_name.clone()
				})).collect();
				content.push(
					pick_list(packs, self.selected_pack.clone(), Message::SelectPack).into()
				);
			} else {
				content.push(text("fetching packs...").into());
			}
		} else {
			match &self.selected_pack {
				Some(selected_pack) => content.push(text(format!("currently selected pack: {}", selected_pack)).into()),
				None => content.push(text("no pack currently selected").into())
			}
		}

		if let Some(mc_path) = &self.mc_path {
			content.push(text(mc_path.display()).into());
		}

		let mut select_button = button("select Minecraft folder");
		if idle {
			select_button = select_button.on_press(Message::SelectMCPath);
		}
		content.push(select_button.into());

		let upgrade_button = button("upgrade");
		if idle && self.packs.is_some() && self.selected_pack.is_some() && self.mc_path.is_some() {
			content.push(upgrade_button.on_press(Message::Upgrade).into());
		} else {
			let mut tooltip_text = String::from("can't upgrade because:");

			tooltip!(tooltip_text, !matches!(self.upgrade_state, UpgradeState::Idle), "currently working on an upgrade");
			tooltip!(tooltip_text, self.packs.is_none(), "currently fetching packs from server");
			tooltip!(tooltip_text, self.selected_pack.is_none(), "you have not selected a pack");
			tooltip!(tooltip_text, !self.can_select_path, "you are currently selecting a Minecraft folder");
			tooltip!(tooltip_text, self.mc_path.is_none(), "you have not selected a Minecraft folder");

			let tooltip = tooltip(upgrade_button, text(tooltip_text), tooltip::Position::FollowCursor)
				.style(theme::Container::Box);
			content.push(tooltip.into());
		}

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
			#[cfg(feature = "cloud-logging")]
			cloud_logging::setup_logging();

			let (tx, mut rx) = mpsc::channel(128);
			output.send(Message::WorkerReady(tx)).await.unwrap();

			{
				let mut output = output.clone();
				tokio::spawn(async move {
					let mut retries = 0;

					let packs_list = loop {
						if let Some(packs) = PacksListManifest::from_url(PACKS_URL).await {
							break packs;
						} else {
							retries += 1;
							if retries >= 3 {
								send_notification("FATAL ERROR: failed to fetch packs from server").await;
								output.send(Message::FailedPackFetch).await.unwrap();
								return;
							}
							log::warn!("failed to fetch packs, retrying");
						}
					};
					output.send(Message::PacksFetched(packs_list)).await.unwrap();
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
				send_notification("upgrade finished").await;
			}

			unreachable!()
		})
	}

	fn theme(&self) -> Theme {
		Theme::Dark
	}
}

fn main() {
	#[cfg(feature = "env-logging")]
	pretty_env_logger::init();

	App::run(Settings {
		window: iced::window::Settings {
			size: iced::Size::new(500.0, 400.0),
			..iced::window::Settings::default()
		},
		..Settings::default()
	}).unwrap();
}
