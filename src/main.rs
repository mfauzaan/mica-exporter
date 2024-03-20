mod storage;
mod mica_handler;

use lambda_http::{run, service_fn, Error};
use mica_handler::extract_layers_and_save;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(extract_layers_and_save)).await
}
