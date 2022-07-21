use gtk::prelude::*;
use relm4::*;
use std::path::PathBuf;

use crate::{AppModel, AppMsg};

pub struct MCSelectModel {
	visible: bool
}

pub enum MCSelectMsg {
	Show,
	Success(PathBuf)
}

impl Model for MCSelectModel {
	type Msg = MCSelectMsg;
	type Widgets = MCSelectWidgets;
	type Components = ();
}

impl ComponentUpdate<AppModel> for MCSelectModel {
	fn init_model(_: &AppModel) -> Self {
		Self {
			visible: false
		}
	}

	fn update(&mut self, msg: MCSelectMsg, _: &(), _: Sender<MCSelectMsg>, parent_sender: Sender<AppMsg>) {
		match msg {
			MCSelectMsg::Show => {
				self.visible = true;
			},
			MCSelectMsg::Success(file) => {
				self.visible = false;
				send!(parent_sender, AppMsg::SetMCPath(file));
			}
		}
	}
}

#[relm4::widget(pub)]
impl Widgets<MCSelectModel, AppModel> for MCSelectWidgets {
	view! {
		gtk::FileChooserNative {
			set_title: "select Minecraft folder",
			set_action: gtk::FileChooserAction::SelectFolder,
			set_transient_for: parent!(Some(&parent_widgets.root_widget())),
			set_visible: watch! { model.visible },
			connect_response(sender) => move |file_chooser, response_type| {
				match response_type {
					gtk::ResponseType::Accept => match file_chooser.file() {
						Some(file) => {
							send!(sender, MCSelectMsg::Success(file.path().unwrap()));
						},
						_ => ()
					},
					_ => ()
				}
			}
		}
	}
}
