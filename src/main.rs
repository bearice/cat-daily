use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Utc};
use dotenv::dotenv;
use egg_mode::{
    entities::{MediaEntity, MediaType},
    tweet::{user_timeline, Tweet},
    KeyPair, Token,
};
use std::env;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    dotenv().ok();
    let user_id: u64 = env::var("user_id")
        .expect("user id not found")
        .parse()
        .unwrap();
    let token = Token::Access {
        consumer: KeyPair::new(
            env::var("consumer_key").expect("consumer key not defined"),
            env::var("consumer_secret").expect("consumer secret not defined"),
        ),
        access: KeyPair::new(
            env::var("access_key").expect("access key not defined"),
            env::var("access_secret").expect("access secret not defined"),
        ),
    };
    let mut cats = load_cats().await.unwrap_or_default();
    let max_id = cats.first().map(|c| c.id);
    let mut home = user_timeline(user_id, false, false, &token).with_page_size(200);
    loop {
        let (new_home, feed) = home.older(max_id).await?;
        if feed.is_empty() || feed[0].created_at.year() < 2020 {
            break;
        }
        eprintln!("Loaded {} tweets from {}", feed.len(), feed[0].created_at);
        for tweet in feed.iter().filter(is_cat_tweet) {
            let cat = CatTweet::from(tweet);
            cats.push(cat);
        }
        home = new_home;
    }
    cats.sort_by(|a, b| b.id.cmp(&a.id));
    save_cats(&cats).await
}

fn is_cat_tweet(tweet: &&Tweet) -> bool {
    tweet
        .entities
        .hashtags
        .iter()
        .any(|hashtag| hashtag.text == "每日一猫")
}

async fn load_cats() -> Result<Vec<CatTweet>> {
    let s = tokio::fs::read("cats.json").await?;
    serde_json::from_slice(&s).context("Failed to parse cats.json")
}

async fn save_cats(cats: &[CatTweet]) -> Result<()> {
    let s = serde_json::to_vec_pretty(cats)?;
    tokio::fs::write("cats.json", s).await?;
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatTweet {
    id: u64,
    text: String,
    created_at: DateTime<Utc>,
    media: Vec<CatMedia>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatMedia {
    media_type: MediaType,
    width: i32,
    height: i32,
    image_url: String,
    video_url: Option<String>,
}

impl From<&Tweet> for CatTweet {
    fn from(tweet: &Tweet) -> Self {
        let media = tweet
            .extended_entities
            .as_ref()
            .map(|media| media.media.iter().map(CatMedia::from).collect())
            .unwrap_or_default();
        CatTweet {
            id: tweet.id,
            text: tweet.text.clone(),
            created_at: tweet.created_at,
            media,
        }
    }
}

impl From<&MediaEntity> for CatMedia {
    fn from(media: &MediaEntity) -> Self {
        CatMedia {
            media_type: media.media_type,
            width: media.sizes.large.w,
            height: media.sizes.large.h,
            image_url: media.media_url.to_string(),
            video_url: media.video_info.as_ref().map(|info| {
                info.variants
                    .iter()
                    .max_by(|x, y| x.bitrate.cmp(&y.bitrate))
                    .map(|variant| variant.url.to_string())
                    .unwrap()
            }),
        }
    }
}

use serde::{Deserialize, Serialize};
use yansi::Paint;
pub fn print_tweet(tweet: &Tweet) {
    if let Some(ref user) = tweet.user {
        println!(
            "{} (@{}) posted at {}",
            Paint::blue(&user.name),
            Paint::bold(Paint::blue(&user.screen_name)),
            tweet.created_at.with_timezone(&chrono::Local)
        );
    }

    // if let Some(ref screen_name) = tweet.in_reply_to_screen_name {
    //     println!("➜ in reply to @{}", Paint::blue(screen_name));
    // }

    // if let Some(ref status) = tweet.retweeted_status {
    //     println!("{}", Paint::red("Retweet ➜"));
    //     print_tweet(status);
    //     return;
    // } else {
    println!("{}", Paint::green(&tweet.text));
    // }

    // if let Some(source) = &tweet.source {
    //     println!("➜ via {} ({})", source.name, source.url);
    // }

    // if let Some(ref place) = tweet.place {
    //     println!("➜ from: {}", place.full_name);
    // }

    // if let Some(ref status) = tweet.quoted_status {
    //     println!("{}", Paint::red("➜ Quoting the following status:"));
    //     print_tweet(status);
    // }

    // if !tweet.entities.hashtags.is_empty() {
    //     println!("➜ Hashtags contained in the tweet:");
    //     for tag in &tweet.entities.hashtags {
    //         println!("  {}", tag.text);
    //     }
    // }

    // if !tweet.entities.symbols.is_empty() {
    //     println!("➜ Symbols contained in the tweet:");
    //     for tag in &tweet.entities.symbols {
    //         println!("  {}", tag.text);
    //     }
    // }

    // if !tweet.entities.urls.is_empty() {
    //     println!("➜ URLs contained in the tweet:");
    //     for url in &tweet.entities.urls {
    //         if let Some(expanded_url) = &url.expanded_url {
    //             println!("  {}", expanded_url);
    //         }
    //     }
    // }

    // if !tweet.entities.user_mentions.is_empty() {
    //     println!("➜ Users mentioned in the tweet:");
    //     for user in &tweet.entities.user_mentions {
    //         println!("  {}", Paint::bold(Paint::blue(&user.screen_name)));
    //     }
    // }

    if let Some(ref media) = tweet.extended_entities {
        println!("➜ Media attached to the tweet:");
        for info in &media.media {
            println!("  A {:?}", info.media_type);
            println!("    URL: {}", info.media_url_https);
            if info.media_type == MediaType::Video {
                let video = info
                    .video_info
                    .as_ref()
                    .unwrap()
                    .variants
                    .iter()
                    .max_by(|x, y| x.bitrate.cmp(&y.bitrate))
                    .unwrap();
                println!("    Video: {}", video.url);
            }
        }
    }
}
