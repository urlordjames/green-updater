pub fn minecraft_path() -> std::path::PathBuf {
	if cfg!(target_os = "windows") {
		todo!("windows implementation");
	} else if cfg!(target_os = "macos") {
		todo!("macos implementation");
	} else {
		let home = std::env::var("HOME").unwrap();
		let home_path = std::path::Path::new(&home);
		home_path.join(".minecraft")
	}
}
