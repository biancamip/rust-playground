use log::info;
use teloxide::{
    prelude::{Requester, ResponseResult},
    types::Message,
    utils::command::BotCommands,
    Bot,
};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", parse_with = "split")]
pub enum LunaBotCommand {
    Start,
    Help,
    Greet,
}

pub async fn handle_message(bot: Bot, msg: Message, cmd: LunaBotCommand) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    match cmd {
        LunaBotCommand::Greet => {
            greet_command(&bot, chat_id).await?;
        }
        LunaBotCommand::Start => {
            start_command(&bot, chat_id, &msg).await?;
        }
        LunaBotCommand::Help => {
            help_command(bot, chat_id).await?;
        }
    }

    if msg.text().is_none() {
        info!(
            "Received message without text from chat {chat_id} with {:?}",
            msg.from
        );
        return Ok(());
    }

    let text = msg.text().unwrap();
    info!("message text: {text}");

    Ok(())
}

async fn help_command(
    bot: Bot,
    chat_id: teloxide::prelude::ChatId,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, "Oops, my tutor got distracted by something and forgot to implement this functionality. I'm meowy sorry!").await?;
    Ok(())
}

async fn start_command(
    bot: &Bot,
    chat_id: teloxide::prelude::ChatId,
    msg: &Message,
) -> Result<(), teloxide::RequestError> {
    // save chat id with user to db

    bot.send_message(
        chat_id,
        "Meow there! I'm Luna, your BFF (Best Feline Friend).",
    )
    .await?;
    info!("Started chat with {:?} - chatid: {}", msg.from, chat_id);
    Ok(())
}

async fn greet_command(
    bot: &Bot,
    chat_id: teloxide::prelude::ChatId,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, "Meowww *rubs head against your hand*")
        .await?;
    info!("Sent Luna's greeting to chat {}", chat_id);
    Ok(())
}
