use itertools::Itertools;
use teloxide::prelude::*;
use teloxide::types::{ChatAction, ParseMode};
use teloxide::utils::html::escape;

use crate::youtube::Youtube;

const COMMANDS: [&str; 4] = ["/cc", "/caps", "/cc@get_ytt_bot", "/caps@get_ytt_bot"];

pub struct BotConfig {
  pub tg_app_url: String,
  pub youtube: Youtube,
}

pub async fn start(cfg: &'static BotConfig) {
  let bot = Bot::from_env();
  let ignore_update = |_upd| Box::pin(async {});
  let handler = Update::filter_message().endpoint(message_handler);
  Dispatcher::builder(bot, handler)
    .default_handler(ignore_update)
    .dependencies(dptree::deps![cfg])
    .build()
    .dispatch()
    .await;
}

async fn message_handler(bot: Bot, msg: Message, cfg: &BotConfig) -> ResponseResult<()> {
  let text = msg.text().unwrap_or("");
  let is_command = COMMANDS.iter().any(|&cmd| text.starts_with(cmd));
  if !is_command && !msg.chat.is_private() {
    return Ok(());
  }

  bot.send_chat_action(msg.chat.id, ChatAction::Typing).await?;
  if is_command {
    bot.delete_message(msg.chat.id, msg.id).await?;
  }

  let quote = match (msg.chat.is_private(), msg.reply_to_message()) {
    (_, Some(quote)) => quote,
    _ => &msg,
  };

  if let Some(id) = quote.text().and_then(Youtube::extract_video_id) {
    let text = get_subs(id, cfg).await.unwrap_or_else(|err| err.to_string());
    bot
      .send_message(msg.chat.id, text)
      .reply_to_message_id(quote.id.into())
      .disable_web_page_preview(true)
      .allow_sending_without_reply(true)
      .parse_mode(ParseMode::Html)
      .await?;
  }

  Ok(())
}

async fn get_subs(id: &str, cfg: &BotConfig) -> youtube_captions::error::Result<String> {
  let caps = cfg.youtube.get_caps_for_video(id).await?;
  let transcript = caps.fetch_srv1().await?;
  let preview = transcript.content.iter().map(|it| it.value.trim()).take(60).join(" ");

  let preview = escape(&preview);
  let tg_app_url = cfg.tg_app_url.as_str();
  let read_next = match caps.language.as_str() {
    "en" => "read further",
    "ky" => "окууну улантуу",
    "ru" => "читать далее",
    "uk" => "читати далі",
    _ => "read further",
    // todo: complete translations
  };

  Ok(format!(r#"{preview}... <b><a href="{tg_app_url}?startapp={id}">{read_next}</a></b>"#))
}
