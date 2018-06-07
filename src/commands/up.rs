use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;

use chrono::Local;
use chrono_humanize::{Accuracy, HumanTime, Tense};
use clap::ArgMatches;
use colored::*;
use reqwest::mime;
use reqwest::multipart::{Form, Part};
use reqwest::Method;
use reqwest::Response;
use serde_json;
use tar;

use api::API;
use app::App;
use commands::common::check_card;
use commands::CommandError;
use config::Config;

pub struct UpCommand;

impl UpCommand {
    fn tar_app(app: &App) -> Result<PathBuf, CommandError> {
        // create tar file
        let path = Config::dir()
            .unwrap()
            .join(format!("build-{}.tar", app.name));
        let f = File::create(&path)
            .map_err(|_| CommandError::with_message("Could not create tar file."))?;

        // write directory contents to tar file
        let mut archive = tar::Builder::new(f);
        archive.append_dir_all(".", ".").map_err(|err| {
            eprintln!("{}", err);
            CommandError::with_message("Could not tar current directory.")
        })?;
        archive
            .finish()
            .map_err(|_| CommandError::with_message("Could not complete tar file."))?;

        Ok(path)
    }

    fn create_form(path: &PathBuf, app: &App) -> Result<Form, CommandError> {
        let raw_app = serde_json::to_string(&app)
            .map_err(|_| CommandError::with_message("Could not write app config to form."))?;
        let app_part = Part::text(raw_app).mime(mime::APPLICATION_JSON);
        Form::new()
            .part("config", app_part)
            .file("file", path)
            .map_err(|_| CommandError::with_message("Could not add tar file to request."))
    }

    fn read_response(res: Response) -> Result<(), CommandError> {
        let reader = io::BufReader::new(res);
        let mut error_lines = vec![];
        let build_prefix = "builder |".blue().bold();
        let start = Local::now();
        for line in reader.lines().map(|l| l.unwrap()) {
            if line.starts_with("[stdout]") {
                println!(
                    "{} {}",
                    build_prefix,
                    line.trim_left_matches("[stdout]").trim_left()
                );
            } else if line.starts_with("[stderr]") {
                eprintln!(
                    "{} {}",
                    build_prefix,
                    line.trim_left_matches("[stderr]").trim_left().red()
                );
            } else if line.starts_with("[result]") {
                let result = line.trim_left_matches("[result]").trim_left();
                if result == "FAILED" {
                    return Err(CommandError::with_message("Failed to deploy."));
                } else {
                    let end = Local::now();
                    let elapsed = HumanTime::from(end - start);
                    println!(
                        "{} {}",
                        "Deployed!".green(),
                        format!(
                            "({})",
                            elapsed.to_text_en(Accuracy::Precise, Tense::Present)
                        ).dimmed()
                    );
                }
            } else {
                error_lines.push(line);
            }
        }

        if error_lines.len() > 0 {
            Err(CommandError::with_message(format!(
                "Failed to deploy:\n{}",
                error_lines.join("\n")
            )))
        } else {
            Ok(())
        }
    }

    pub fn run(_matches: &ArgMatches, config: &Config, app: &App) -> Result<(), CommandError> {
        check_card(config)?;

        let path = UpCommand::tar_app(app)?;
        let form = UpCommand::create_form(&path, app)?;
        let res = API::new(config)
            .apps()
            .param(&format!("{}/up", app.name))
            .request(Method::Post)
            .multipart(form)
            .send()?;

        UpCommand::read_response(res)
    }
}
