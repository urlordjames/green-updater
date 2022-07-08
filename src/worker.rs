use relm4::*;

use crate::{AppModel, AppMsg};

pub struct WorkerModel {}

pub enum WorkerMsg {
	Upgrade((url::Url, std::path::PathBuf))
}

impl Model for WorkerModel {
	type Msg = WorkerMsg;
	type Widgets = ();
	type Components = ();
}

#[async_trait]
impl AsyncComponentUpdate<AppModel> for WorkerModel {
	fn init_model(_: &AppModel) -> Self {
		WorkerModel {}
	}

	async fn update(&mut self, msg: WorkerMsg, _: &(), _: Sender<WorkerMsg>, parent_sender: Sender<AppMsg>) {
		match msg {
			WorkerMsg::Upgrade((url, download_path)) => {
				let remote_directory = green_lib::Directory::from_url(url).await;

				match remote_directory {
					Some(remote_directory) => {
						println!("valid manifest, attempting upgrade...");
						remote_directory.upgrade_game_folder(&download_path).await;

						println!("successfully upgraded game folder");
						send!(parent_sender, AppMsg::UnlockUpgrade);
					},
					None => panic!("invalid manifest")
				}
			}
		};
	}
}
