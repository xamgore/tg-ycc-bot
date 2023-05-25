use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use youtube_captions::language_tags::LanguageTag;
use youtube_captions::{CaptionScraper, DigestScraper, Result};

pub struct Youtube {
  scraper: DigestScraper,
}

static YT_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
  // https://stackoverflow.com/a/51870158/3160483
  Regex::new(r#"(?i)(?:https?://)?(?:(?:(?:m|www)\.)?(?:youtube(?:-nocookie)?|youtube.googleapis)\.com.*(?:v/|v=|vi=|vi/|e/|embed/|user/.*/u/\d+/)|youtu\.be/)([_0-9a-z-]+)"#).unwrap()
});

impl Youtube {
  pub fn new(http: reqwest::Client) -> Self {
    Self { scraper: DigestScraper::new(http) }
  }

  pub fn extract_video_id(text: &str) -> Option<&str> {
    YT_ID_REGEX.captures(text).and_then(|caps| caps.get(1)).map(|mat| mat.as_str())
  }

  pub async fn get_caps_for_video(&self, id: &str) -> Result<CaptionScraper> {
    let digest = self.scraper.fetch(id, "en").await?;

    let tags = ["ru", "en", "uk", "ky"] // todo: move to arguments
      .into_iter()
      .map(|lang| LanguageTag::parse(lang).unwrap())
      .collect_vec();

    let caption = tags
      .iter()
      .flat_map(|lang| digest.captions.iter().filter(|cap| lang.matches(&cap.lang_tag)).cloned())
      .sorted_by(|a, b| a.is_generated.cmp(&b.is_generated))
      .next()
      .unwrap();

    Ok(caption)
  }
}
