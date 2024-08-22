use tracing_subscriber::filter::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;

pub fn setup_logging() -> WorkerGuard {
	let mut log_dir = std::env::temp_dir();
	log_dir.push("green_updater");
	eprintln!("file logging is enabled, writing logs to {:?}", &log_dir);

	let file_appender = tracing_appender::rolling::daily(log_dir, "green_updater.log");
	let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

	tracing_subscriber::fmt()
		.with_max_level(LevelFilter::DEBUG)
		.with_ansi(false)
		.with_writer(non_blocking)
		.init();

	std::panic::set_hook(Box::new(tracing_panic::panic_hook));

	guard
}
