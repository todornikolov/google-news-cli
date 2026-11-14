//! Google News, straight from Google's public RSS feed. No key, no account.

use std::cmp::Reverse;
use std::time::{Duration, SystemTime};

use rss::Channel;

use crate::{Hit, Period, TOP};

const FEED: &str = "https://news.google.com/rss/search";

/// How to narrow and order a search.
pub struct Options {
    /// Only stories from this long ago until now.
    pub since: Option<Period>,
    /// Newest first, instead of Google's own ranking.
    pub newest_first: bool,
}

pub fn search(
    query: &str,
    country: &str,
    lang: &str,
    options: &Options,
) -> anyhow::Result<Vec<Hit>> {
    let country = country.to_uppercase();

    let xml = ureq::get(FEED)
        .query("q", full_query(query, options.since))
        .query("hl", hl(lang, &country))
        .query("gl", &country)
        .query("ceid", format!("{country}:{lang}"))
        .config()
        .timeout_global(Some(Duration::from_secs(20)))
        .build()
        .call()?
        .body_mut()
        .read_to_string()?;

    parse(&xml, options.newest_first)
}

/// Google's own feed links use "en-US" style for English and just "de", "bg"... otherwise.
fn hl(lang: &str, country: &str) -> String {
    match lang {
        "en" => format!("en-{country}"),
        _ => lang.to_owned(),
    }
}

/// Google filters by time through a `when:` operator inside the query itself.
fn full_query(query: &str, since: Option<Period>) -> String {
    match since {
        Some(period) => format!("{query} {}", period.operator()),
        None => query.to_owned(),
    }
}

fn parse(xml: &str, newest_first: bool) -> anyhow::Result<Vec<Hit>> {
    let channel: Channel = xml.parse()?;
    let mut items: Vec<&rss::Item> = channel.items().iter().collect();
    // Sort before taking the top ten, so we get the ten newest, not the newest of the first ten.
    // The sort is stable and `None` sorts lowest, so undated items go last in feed order.
    if newest_first {
        items.sort_by_key(|item| Reverse(published(item)));
    }
    Ok(items.into_iter().filter_map(to_hit).take(TOP).collect())
}

fn published(item: &rss::Item) -> Option<SystemTime> {
    httpdate::parse_http_date(item.pub_date()?).ok()
}

fn to_hit(item: &rss::Item) -> Option<Hit> {
    let title = item.title()?;
    let link = item.link()?;
    let source = item.source().and_then(|s| s.title()).unwrap_or_default();
    let date = item.pub_date().map(short_date).unwrap_or_default();

    Some(Hit {
        title: headline(title, source).to_owned(),
        link: link.to_owned(),
        detail: detail(source, date),
    })
}

/// Titles arrive as "Headline - Publisher"; the publisher goes in the detail line instead.
fn headline<'a>(title: &'a str, source: &str) -> &'a str {
    title.strip_suffix(&format!(" - {source}")).unwrap_or(title)
}

/// "Fri, 14 Nov 2026 09:30:00 GMT" -> "Fri, 14 Nov 2026"
fn short_date(date: &str) -> &str {
    date.get(..16).unwrap_or(date)
}

/// "Publisher · date", leaving out whichever is missing.
fn detail(source: &str, date: &str) -> String {
    [source, date]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0"><channel>
  <title>"rust" - Google News</title>
  <link>https://news.google.com/search?q=rust</link>
  <description>Google News</description>
  <item>
    <title>Rust 1.92 released - The Register</title>
    <link>https://news.google.com/rss/articles/CBMiAbc?oc=5</link>
    <pubDate>Fri, 14 Nov 2026 09:30:00 GMT</pubDate>
    <source url="https://www.theregister.com">The Register</source>
  </item>
  <item>
    <title>No source on this one</title>
    <link>https://news.google.com/rss/articles/CBMiDef?oc=5</link>
  </item>
</channel></rss>"#;

    #[test]
    fn moves_publisher_from_title_to_detail() {
        let hits = parse(SAMPLE, false).unwrap();
        assert_eq!(
            hits[0],
            Hit {
                title: "Rust 1.92 released".into(),
                link: "https://news.google.com/rss/articles/CBMiAbc?oc=5".into(),
                detail: "The Register · Fri, 14 Nov 2026".into(),
            }
        );
    }

    #[test]
    fn copes_with_missing_source_and_date() {
        let hits = parse(SAMPLE, false).unwrap();
        assert_eq!(hits[1].title, "No source on this one");
        assert_eq!(hits[1].detail, "");
    }

    #[test]
    fn keeps_only_the_top_ten() {
        let item = "<item><title>t</title><link>https://example.com</link></item>";
        let xml = format!(
            "<rss version=\"2.0\"><channel><title>x</title><link>l</link><description>d</description>{}</channel></rss>",
            item.repeat(15)
        );
        assert_eq!(parse(&xml, false).unwrap().len(), 10);
    }

    #[test]
    fn newest_first_orders_by_date_and_puts_undated_last() {
        let item = |title: &str, date: &str| {
            let pub_date = if date.is_empty() {
                String::new()
            } else {
                format!("<pubDate>{date}</pubDate>")
            };
            format!(
                "<item><title>{title}</title><link>https://example.com/{title}</link>{pub_date}</item>"
            )
        };
        let xml = format!(
            "<rss version=\"2.0\"><channel><title>x</title><link>l</link><description>d</description>{}{}{}{}</channel></rss>",
            item("old", "Tue, 10 Nov 2026 08:00:00 GMT"),
            item("undated", ""),
            item("new", "Sat, 14 Nov 2026 09:30:00 GMT"),
            item("mid", "Thu, 12 Nov 2026 12:00:00 GMT"),
        );

        let titles = |newest_first| -> Vec<String> {
            parse(&xml, newest_first)
                .unwrap()
                .into_iter()
                .map(|hit| hit.title)
                .collect()
        };

        assert_eq!(titles(false), ["old", "undated", "new", "mid"]);
        assert_eq!(titles(true), ["new", "mid", "old", "undated"]);
    }

    #[test]
    fn newest_first_picks_the_ten_newest_not_the_first_ten() {
        // 1 Nov 2026 is a Sunday, and the parser checks that weekday names match the date.
        const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let item = |day: usize| {
            let weekday = WEEKDAYS[(day - 1) % 7];
            format!(
                "<item><title>day {day}</title><link>https://example.com/{day}</link><pubDate>{weekday}, {day:02} Nov 2026 00:00:00 GMT</pubDate></item>"
            )
        };
        let items: String = (1..=15).map(item).collect();
        let xml = format!(
            "<rss version=\"2.0\"><channel><title>x</title><link>l</link><description>d</description>{items}</channel></rss>"
        );

        let hits = parse(&xml, true).unwrap();

        assert_eq!(hits[0].title, "day 15");
        assert_eq!(hits[9].title, "day 6");
    }

    #[test]
    fn a_time_period_adds_googles_when_operator_to_the_query() {
        assert_eq!(full_query("rust", None), "rust");
        assert_eq!(full_query("rust", Some(Period::Hour)), "rust when:1h");
        assert_eq!(full_query("rust", Some(Period::Day)), "rust when:1d");
        assert_eq!(full_query("rust", Some(Period::Week)), "rust when:7d");
        assert_eq!(full_query("rust", Some(Period::Month)), "rust when:30d");
    }

    #[test]
    fn english_uses_the_en_us_style_of_googles_own_links() {
        assert_eq!(hl("en", "US"), "en-US");
        assert_eq!(hl("en", "GB"), "en-GB");
    }

    #[test]
    fn other_languages_stand_alone() {
        assert_eq!(hl("bg", "BG"), "bg");
        assert_eq!(hl("de", "AT"), "de");
    }

    /// Wraps a title and optional extras in a one-item feed and returns its only hit.
    fn only_hit(title: &str, extras: &str) -> Hit {
        let xml = format!(
            "<rss version=\"2.0\"><channel><title>x</title><link>l</link><description>d</description>\
             <item><title>{title}</title><link>https://example.com</link>{extras}</item></channel></rss>"
        );
        parse(&xml, false).unwrap().remove(0)
    }

    #[test]
    fn keeps_a_title_whole_when_it_does_not_end_with_the_publisher() {
        let hit = only_hit(
            "Rust 1.92 - what's new",
            r#"<source url="https://x.example">The Register</source>"#,
        );

        assert_eq!(hit.title, "Rust 1.92 - what's new");
        assert_eq!(hit.detail, "The Register");
    }

    #[test]
    fn shows_only_the_weekday_day_month_and_year_of_the_date() {
        let hit = only_hit(
            "Headline",
            "<pubDate>Sat, 14 Nov 2026 09:30:00 GMT</pubDate>",
        );

        assert_eq!(hit.detail, "Sat, 14 Nov 2026");
    }

    #[test]
    fn shows_a_short_date_as_it_is() {
        let hit = only_hit("Headline", "<pubDate>yesterday</pubDate>");

        assert_eq!(hit.detail, "yesterday");
    }

    #[test]
    fn rejects_non_rss() {
        assert!(parse("<html>Sorry, something went wrong</html>", false).is_err());
    }
}
