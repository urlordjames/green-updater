const NOTIFICATION_TITLE: &str = "Green Updater";

#[cfg(target_os = "linux")]
pub async fn send_notification(body: &'static str) {
	use ashpd::desktop::notification::{Notification, NotificationProxy};

	if let Ok(proxy) = NotificationProxy::new().await {
		let _ = proxy.add_notification("io.github.urlordjames.GreenUpdater",
			Notification::new(NOTIFICATION_TITLE)
				.body(body)
		).await;
	}
}

#[cfg(target_os = "windows")]
pub async fn send_notification(body: &'static str) {
	use tauri_winrt_notification::{Toast, Duration, Scenario};

	tokio::task::spawn_blocking(move || {
		let _ = Toast::new(Toast::POWERSHELL_APP_ID)
			.title(NOTIFICATION_TITLE)
			.text1(body)
			.duration(Duration::Short)
			.scenario(Scenario::Reminder)
			.show();
	}).await.unwrap();
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub async fn send_notification(_body: &'static str) {
	// not implemented
}
