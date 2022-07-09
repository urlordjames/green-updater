pub fn minecraft_path() -> std::path::PathBuf {
	if cfg!(target_os = "windows") {
		println!("WARNING: STUB IMPLEMENTATION OF WINDOWS MINECRAFT PATH!");
		std::path::PathBuf::from("C:/fake_dir")
	} else if cfg!(target_os = "macos") {
		todo!("macos implementation");
	} else {
		let home = std::env::var("HOME").unwrap();
		let home_path = std::path::Path::new(&home);
		home_path.join(".minecraft")
	}
}
