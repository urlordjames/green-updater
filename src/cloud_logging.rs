use aws_config::{SdkConfig, BehaviorVersion};
use aws_types::region::Region;
use aws_types::sdk_config::SharedCredentialsProvider;
use aws_credential_types::Credentials;
use aws_credential_types::provider::ProvideCredentials as ProvideCredentialsTrait;
use aws_credential_types::provider::future::ProvideCredentials;
use aws_credential_types::provider::Result as CredsResult;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// these are not secret because it ONLY has permissions to add new logs
const ACCESS_KEY_ID: &str = "AKIA2ZP5AP3Z6LLITZZ6";
const ACCESS_KEY_SECRET: &str = "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";

#[derive(Debug)]
struct CredentialProvider;

impl ProvideCredentialsTrait for CredentialProvider {
	fn provide_credentials<'a>(&'a self) -> ProvideCredentials<'a>
		where Self: 'a
	{
		ProvideCredentials::ready(CredsResult::Ok(Credentials::from_keys(
			ACCESS_KEY_ID,
			ACCESS_KEY_SECRET,
			None
		)))
	}
}

pub fn setup_logging() {
	let config = SdkConfig::builder()
		.region(Region::new("us-east-2"))
		.credentials_provider(SharedCredentialsProvider::new(CredentialProvider))
		.behavior_version(BehaviorVersion::latest())
		.build();

	let client = aws_sdk_cloudwatchlogs::Client::new(&config);

	tracing_subscriber::registry::Registry::default()
		.with(tracing_cloudwatch::layer().with_client(
			client,
			tracing_cloudwatch::ExportConfig::default()
				.with_batch_size(5)
				.with_interval(std::time::Duration::from_secs(5))
				.with_log_group_name("green_updater")
				.with_log_stream_name("the_mega_stream") // this will do for now
			)
			.with_code_location(true)
			.with_target(false)
		).init();
}
