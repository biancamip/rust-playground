use log::info;
use telegram_bot::{handle_message, LunaBotCommand};
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    info!("Starting Luna's bot...");

    let bot = Bot::from_env();
    LunaBotCommand::repl(bot, handle_message).await;
}
