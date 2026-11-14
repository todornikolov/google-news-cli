//! Google News CLI (`gnews`): the top ten Google News results for a query, in your terminal.

mod news;
mod ui;

use clap::{Parser, ValueEnum};

/// How many results to show.
const TOP: usize = 10;

/// One result. This is all the UI needs.
#[derive(Debug, PartialEq, Eq)]
pub struct Hit {
    pub title: String,
    pub link: String,
    pub detail: String,
}

/// How far back to look.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Period {
    Hour,
    Day,
    Week,
    Month,
}

impl Period {
    /// Google News's own query operator for this period.
    pub fn operator(self) -> &'static str {
        match self {
            Self::Hour => "when:1h",
            Self::Day => "when:1d",
            Self::Week => "when:7d",
            Self::Month => "when:30d",
        }
    }
}

/// Top ten Google News results in your terminal. No key or account needed.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// What to search for (quotes optional)
    #[arg(required = true)]
    query: Vec<String>,

    /// Only show stories from the last hour, day, week or month
    #[arg(short, long, value_enum)]
    since: Option<Period>,

    /// Use Google's own ranking instead of showing the newest stories first
    #[arg(long)]
    relevance: bool,

    /// Country to search from, e.g. us, gb, bg
    #[arg(long, default_value = "us")]
    country: String,

    /// Language of the results, e.g. en, bg
    #[arg(long, default_value = "en")]
    lang: String,
}

impl Args {
    /// Newest first unless `--relevance` asks for Google's own ranking.
    fn options(&self) -> news::Options {
        news::Options {
            since: self.since,
            newest_first: !self.relevance,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let query = args.query.join(" ");

    eprintln!("Searching Google News for \"{query}\"...");
    let hits = news::search(&query, &args.country, &args.lang, &args.options())?;

    if hits.is_empty() {
        println!("No results.");
        return Ok(());
    }

    let app = ui::App::new(format!("Google News · {query}"), hits);
    ratatui::run(|terminal| app.run(terminal))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Args, clap::Error> {
        Args::try_parse_from(std::iter::once("gnews").chain(args.iter().copied()))
    }

    #[test]
    fn the_command_is_called_gnews() {
        use clap::CommandFactory;

        assert_eq!(Args::command().get_name(), "gnews");
    }

    #[test]
    fn the_query_can_be_several_words_without_quotes() {
        let args = parse(&["rust", "tui", "crates"]).unwrap();

        assert_eq!(args.query.join(" "), "rust tui crates");
    }

    #[test]
    fn a_query_is_required() {
        assert!(parse(&[]).is_err());
        assert!(parse(&["--since", "day"]).is_err());
    }

    #[test]
    fn defaults_to_us_english_with_no_period() {
        let args = parse(&["rust"]).unwrap();

        assert_eq!(args.country, "us");
        assert_eq!(args.lang, "en");
        assert_eq!(args.since, None);
    }

    #[test]
    fn newest_first_is_the_default() {
        let options = parse(&["rust"]).unwrap().options();

        assert!(options.newest_first);
    }

    #[test]
    fn relevance_switches_back_to_googles_ranking() {
        let options = parse(&["--relevance", "rust"]).unwrap().options();

        assert!(!options.newest_first);
    }

    #[test]
    fn the_old_recent_flag_is_gone() {
        assert!(parse(&["--recent", "rust"]).is_err());
        assert!(parse(&["-r", "rust"]).is_err());
    }

    #[test]
    fn since_is_passed_on_to_the_search_whatever_the_ordering() {
        let newest = parse(&["-s", "hour", "rust"]).unwrap().options();
        let ranked = parse(&["--relevance", "-s", "hour", "rust"])
            .unwrap()
            .options();

        assert_eq!(newest.since, Some(Period::Hour));
        assert_eq!(ranked.since, Some(Period::Hour));
    }

    #[test]
    fn since_accepts_each_period_in_long_and_short_form() {
        for (word, period) in [
            ("hour", Period::Hour),
            ("day", Period::Day),
            ("week", Period::Week),
            ("month", Period::Month),
        ] {
            assert_eq!(
                parse(&["--since", word, "rust"]).unwrap().since,
                Some(period)
            );
            assert_eq!(parse(&["-s", word, "rust"]).unwrap().since, Some(period));
        }
    }

    #[test]
    fn since_rejects_an_unknown_period() {
        assert!(parse(&["--since", "year", "rust"]).is_err());
    }

    #[test]
    fn country_and_language_can_be_overridden() {
        let args = parse(&["--country", "bg", "--lang", "bg", "софия"]).unwrap();

        assert_eq!((args.country.as_str(), args.lang.as_str()), ("bg", "bg"));
        assert_eq!(args.query, ["софия"]);
    }
}
