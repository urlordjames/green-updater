use relm4::*;

use crate::{AppModel, AppMsg};

pub struct WorkerModel;

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
		Self
	}

	async fn update(&mut self, msg: WorkerMsg, _: &(), _: Sender<WorkerMsg>, parent_sender: Sender<AppMsg>) {
		match msg {
			WorkerMsg::Upgrade((url, download_path)) => {
				let remote_directory = green_lib::Directory::from_url(url).await;

				match remote_directory {
					Some(remote_directory) => {
						println!("valid manifest, attempting upgrade...");

						let (tx, mut rx) = tokio::sync::mpsc::channel(128);
						let handle = tokio::spawn(async move {
							remote_directory.upgrade_game_folder(&download_path, Some(tx)).await;
						});

						while let Some(msg) = rx.recv().await {
							match msg {
								green_lib::UpgradeStatus::Tick => {
									send!(parent_sender, AppMsg::Tick)
								},
								green_lib::UpgradeStatus::Length(size) => {
									send!(parent_sender, AppMsg::Total(size));
								}
							}
						}

						handle.await.unwrap();

						println!("successfully upgraded game folder");
						send!(parent_sender, AppMsg::FinishedUpgrade);
					},
					None => panic!("invalid manifest")
				}
			}
		};
	}
}
