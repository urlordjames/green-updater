use gtk::prelude::*;
use relm4::*;

use crate::{AppModel, AppMsg};

pub struct FinishedModel {
	visible: bool
}

pub enum FinishedMsg {
	Finished,
	Dismiss
}

impl Model for FinishedModel {
	type Msg = FinishedMsg;
	type Widgets = FinishedWidgets;
	type Components = ();
}

impl ComponentUpdate<AppModel> for FinishedModel {
	fn init_model(_: &AppModel) -> Self {
		Self {
			visible: false
		}
	}

	fn update(&mut self, msg: FinishedMsg, _: &(), _: Sender<FinishedMsg>, parent_sender: Sender<AppMsg>) {
		match msg {
			FinishedMsg::Finished => {
				self.visible = true;
			},
			FinishedMsg::Dismiss => {
				self.visible = false;
				send!(parent_sender, AppMsg::FinishDismissed);
			}
		}
	}
}

#[relm4::widget(pub)]
impl Widgets<FinishedModel, AppModel> for FinishedWidgets {
	view! {
		gtk::MessageDialog {
			set_title: Some("Green Updater - finished"),
			set_text: Some("upgrade finished"),
			set_visible: watch!{ model.visible },
			set_transient_for: parent!(Some(&parent_widgets.root_widget())),
			set_deletable: false,
			add_button: args!("dismiss", gtk::ResponseType::Close),
			connect_response(sender) => move |_, resp| {
				match resp {
					gtk::ResponseType::Close => send!(sender, FinishedMsg::Dismiss),
					_ => unreachable!()
				}
			}
		}
	}
}
