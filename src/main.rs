#![windows_subsystem = "windows"]

use gtk::prelude::*;
use relm4::*;
use std::path::PathBuf;

mod mc_select;
use mc_select::{MCSelectModel, MCSelectMsg};

mod worker;
use worker::{WorkerModel, WorkerMsg};

mod finished;
use finished::{FinishedModel, FinishedMsg};

use green_lib::util;

struct AppModel {
	url: url::Url,
	mc_path: PathBuf,
	buttons_work: bool,
	spinning: bool
}

enum AppMsg {
	Open,
	SetMCPath(PathBuf),
	Upgrade,
	FinishedUpgrade,
	FinishDismissed
}

impl Model for AppModel {
	type Msg = AppMsg;
	type Widgets = AppWidgets;
	type Components = AppComponents;
}

impl AppUpdate for AppModel {
	fn update(&mut self, msg: AppMsg, components: &AppComponents, sender: Sender<AppMsg>) -> bool {
		match msg {
			AppMsg::Open => {
				send!(components.mc_select, MCSelectMsg::Show);
			},
			AppMsg::SetMCPath(path) => {
				self.mc_path = path;
			},
			AppMsg::Upgrade => {
				self.buttons_work = false;
				self.spinning = true;
				send!(components.worker, WorkerMsg::Upgrade((self.url.clone(), self.mc_path.clone())));
			},
			AppMsg::FinishedUpgrade => {
				self.spinning = false;

				match notify_rust::Notification::new()
					.summary("green updater finished upgrade")
					.show() {
					Ok(handle) => {
						handle.on_close(|| send!(sender, AppMsg::FinishDismissed));
					},
					Err(_) => {
						send!(components.finished, FinishedMsg::Finished);
					}
				};
			},
			AppMsg::FinishDismissed => {
				self.buttons_work = true;
			}
		};

		true
	}
}

#[relm4::widget]
impl Widgets<AppModel, ()> for AppWidgets {
	view! {
		gtk::ApplicationWindow {
			set_title: Some("green-updater"),
			set_child = Some(&gtk::Box) {
				set_orientation: gtk::Orientation::Vertical,

				append = &gtk::Label {
					set_markup: "<a href=\"https://github.com/urlordjames/green-updater\">green updater</a>\n(licensed under GPL-3.0 or later)",
					set_wrap: true,
					set_justify: gtk::Justification::Center,
					set_margin_bottom: 25
				},
				append = &gtk::Label {
					set_label: "select a Minecraft folder\n(you can ignore this if you use the default Minecraft installer)",
					set_wrap: true,
					set_justify: gtk::Justification::Center
				},
				append = &gtk::Button {
					set_label: "open",
					connect_clicked(sender) => move |_| {
						send!(sender, AppMsg::Open);
					},
					set_sensitive: watch! { model.buttons_work }
				},
				append = &gtk::Label {
					set_label: watch! { &format!("target Minecraft folder: {:?}", model.mc_path) }
				},
				append = &gtk::Button {
					set_label: "upgrade",
					connect_clicked(sender) => move |_| {
						send!(sender, AppMsg::Upgrade);
					},
					set_sensitive: watch! { model.buttons_work }
				},
				append = &gtk::Spinner {
					set_spinning: watch! { model.spinning }
				}
			}
		}
	}
}

struct AppComponents {
	mc_select: RelmComponent<MCSelectModel, AppModel>,
	worker: AsyncRelmWorker<WorkerModel, AppModel>,
	finished: RelmComponent<FinishedModel, AppModel>
}

impl Components<AppModel> for AppComponents {
	fn init_components(parent_model: &AppModel, parent_sender: Sender<AppMsg>) -> Self {
		Self {
			mc_select: RelmComponent::new(parent_model, parent_sender.clone()),
			worker: AsyncRelmWorker::with_new_tokio_rt(parent_model, parent_sender.clone()),
			finished: RelmComponent::new(parent_model, parent_sender)
		}
	}

	fn connect_parent(&mut self, parent_widgets: &AppWidgets) {
		self.mc_select.connect_parent(parent_widgets);
		self.finished.connect_parent(parent_widgets);
	}
}

fn main() {
	let model = AppModel {
		url: url::Url::parse("https://s3-us-east-2.amazonaws.com/le-mod-bucket/manifest.json").unwrap(),
		mc_path: util::minecraft_path(),
		buttons_work: true,
		spinning: false
	};

	let app = RelmApp::new(model);
	app.run();
}
