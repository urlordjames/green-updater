#[cfg(target_os = "linux")]
pub async fn notify_upgrade_done() {
	todo!("implement using zbus")
}

#[cfg(target_os = "windows")]
pub async fn notify_upgrade_done() {
	use tauri_winrt_notification::{Toast, Duration, Scenario};

	tokio::task::spawn_blocking(move || {
		Toast::new(Toast::POWERSHELL_APP_ID)
			.title("Green Updater")
			.text1("upgrade finished")
			.duration(Duration::Short)
			.scenario(Scenario::Reminder)
			.show().unwrap();
	}).await.unwrap();
}