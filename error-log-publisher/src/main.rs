use std::{
    thread,
    time::{Duration, Instant},
};

use log::{error, info};
use service_logger::ServiceLogger;

fn main() {
    dotenv::dotenv().ok();
    ServiceLogger::init_from_env();

    let now = Instant::now();

    let tokio_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to initialize tokio runtime");

    let future = tokio_rt.block_on(async move { main_async().await });

    if let Err(err) = future {
        error!("{:?}", err);
        std::thread::sleep(Duration::from_secs(5));
        std::process::exit(1);
    }

    info!("total time: {}s", now.elapsed().as_secs());
}

async fn main_async() -> anyhow::Result<()> {
    error!("hi! this is an error log. did i show up?");
    thread::sleep(Duration::from_secs(15));
    Ok(())
}
