const NOTIFICATION_TITLE: &str = "Green Updater";
const NOTIFICATION_BODY: &str = "upgrade finished";

#[cfg(target_os = "linux")]
pub async fn notify_upgrade_done() {
	use ashpd::desktop::notification::{Notification, NotificationProxy};

	if let Ok(proxy) = NotificationProxy::new().await {
		let _ = proxy.add_notification("io.github.urlordjames.GreenUpdater",
			Notification::new(NOTIFICATION_TITLE)
				.body(NOTIFICATION_BODY)
		).await;
	}
}

#[cfg(target_os = "windows")]
pub async fn notify_upgrade_done() {
	use tauri_winrt_notification::{Toast, Duration, Scenario};

	tokio::task::spawn_blocking(move || {
		let _ = Toast::new(Toast::POWERSHELL_APP_ID)
			.title(NOTIFICATION_TITLE)
			.text1(NOTIFICATION_BODY)
			.duration(Duration::Short)
			.scenario(Scenario::Reminder)
			.show();
	}).await.unwrap();
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub async fn notify_upgrade_done() {
	// not implemented
}
