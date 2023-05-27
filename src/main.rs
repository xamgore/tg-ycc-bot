use crate::bot::BotConfig;
use crate::youtube::Youtube;

mod bot;
mod server;
mod youtube;

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  dotenvy::dotenv()?;
  tracing_subscriber::fmt::init();

  let cfg: &'static BotConfig = Box::leak(Box::new(BotConfig {
    tg_app_url: std::env::var("TG_APP_URL").unwrap(),
    youtube: Youtube::new(reqwest::Client::new()),
  }));

  futures::future::join(server::start(cfg), bot::start(cfg)).await;
  Ok(())
}
