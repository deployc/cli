use std::io::{self, BufRead};

use chrono::{DateTime, Local};
use clap::ArgMatches;
use colored::*;
use regex::Regex;
use reqwest::{Method, Response};

use api::API;
use app::App;
use commands::CommandError;
use config::Config;

lazy_static! {
    static ref ISODATE_STRING_RE: Regex = Regex::new(
        r"^\d{4}-[01]\d-[0-3]\dT[0-2]\d:[0-5]\d:[0-5]\d(\.\d+)?([+-][0-2]\d:[0-5]\d|Z)"
    ).unwrap();
}

pub struct LogsCommand;

impl LogsCommand {
    fn read_response(res: Response) -> Result<(), CommandError> {
        let reader = io::BufReader::new(res);
        for line in reader.lines().map(|l| l.unwrap()) {
            let v: Vec<_> = line.splitn(2, '|').collect();
            match v[..] {
                [prefix, line] => {
                    let prefix = format!("{} |", prefix);
                    if let Some(mat) = ISODATE_STRING_RE.find(line) {
                        let isostr = mat.as_str();
                        // TODO: Allow user to customize the format of this
                        let ts = format!(
                            "[{}]",
                            DateTime::parse_from_rfc3339(isostr)
                                .unwrap()
                                .with_timezone(&Local)
                                .format("%a, %e %b %Y %H:%M:%S")
                        );
                        println!(
                            "{} {} {}",
                            prefix.blue().bold(),
                            ts.green().bold(),
                            line.get(mat.end() + 1..).unwrap()
                        );
                    } else {
                        println!("{} {}", prefix.blue().bold(), line);
                    }
                }
                _ => continue,
            }
        }

        Ok(())
    }

    pub fn run(matches: &ArgMatches, config: &Config, app: &App) -> Result<(), CommandError> {
        let follow = if matches.is_present("follow") {
            "1"
        } else {
            "0"
        };

        let mut query: Vec<(&str, String)> = vec![("follow", follow.to_string())];
        if let Ok(lines) = value_t!(matches, "tail", u32) {
            query.push(("tail", format!("{}", lines)));
        }

        let res = API::new(config)
            .apps()
            .param(&format!("{}/logs", app.name))
            .request(Method::Get)
            .query(&query)
            .send()?;

        LogsCommand::read_response(res)
    }
}
