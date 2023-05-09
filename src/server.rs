use std::fmt::{Display, Formatter};

use itertools::Itertools;
use poem::listener::TcpListener;
use poem::web::{Data, Html, Query};
use poem::{handler, EndpointExt, Route, Server};
use sailfish::TemplateOnce;
use serde::Deserialize;
use youtube_captions::format::srv1::Transcript;

use crate::bot::BotConfig;

pub async fn start(cfg: &'static BotConfig) {
  let app = Route::new().at("/*path", index).data(cfg);
  Server::new(TcpListener::bind("0.0.0.0:53899")).name("tg-ycc-bot").run(app).await.unwrap();
}

#[derive(Debug, Deserialize)]
struct WebAppParams {
  #[serde(rename = "tgWebAppStartParam")]
  id: String,
}

#[handler]
async fn index(query: Query<WebAppParams>, cfg: Data<&&BotConfig>) -> anyhow::Result<Html<String>> {
  let transcript = cfg.youtube.get_caps_for_video(&query.id).await?.fetch_srv1().await?;
  let template = Template { _id: &query.id, _groups: group_close_captions(transcript) };
  Ok(Html(template.render_once()?))
}

struct Time(u64);

impl Display for Time {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if self.0 >= 3600 {
      write!(f, "{}:{:02}:{:02}", self.0 / 3600, self.0 / 60 % 60, self.0 % 60)
    } else {
      write!(f, "{:02}:{:02}", self.0 / 60, self.0 % 60)
    }
  }
}

fn group_close_captions(transcript: Transcript) -> Vec<(Time, String)> {
  let groups = transcript
    .content
    .into_iter()
    .scan((0f32, f32::MIN), |(idx, end), it| {
      if it.start > *end {
        *idx = it.start;
      }
      *end = it.end().max(*end);
      Some((*idx, it))
    })
    .group_by(|(idx, _)| *idx);

  groups
    .into_iter()
    .map(|(key, group)| (Time(key as u64), group.map(|g| g.1.value).join(" ")))
    .collect_vec()
}

#[derive(TemplateOnce)]
#[template(path = "template.html")]
struct Template<'a> {
  _id: &'a str,
  _groups: Vec<(Time, String)>,
}
