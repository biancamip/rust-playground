use std::time::Instant;

use log::info;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let now = Instant::now();

    info!("Starting main async...");
    let result = main_async().await;

    info!("Total running time: {}ms", now.elapsed().as_millis());

    if let Err(error) = result {
        info!("Task panicked with error: {:?}", error);
        std::process::exit(1);
    }
}

async fn main_async() -> anyhow::Result<()> {
    info!("Starting throw dice bot...");

    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_dice(msg.chat.id).await?;
        Ok(())
    })
    .await;

    Ok(())
}
