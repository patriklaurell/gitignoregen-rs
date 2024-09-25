use clap::{Parser, Subcommand};
use ctrlc; // Add this import
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Select};
use reqwest;
use std::collections::HashSet;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use termion::cursor::Show; // Add this import
use tokio; // Add this import

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Append,
}

const API_URL: &str = "https://www.toptal.com/developers/gitignore/api";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Handle append command
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Append) => {
            let content = get_gitignore_content(true).await?;
            if content.is_none() {
                return Ok(());
            }
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(".gitignore")?;
            file.write_all(content.unwrap().as_bytes())?;
        }
        None => {
            let content = get_gitignore_content(false).await?;
            if content.is_none() {
                return Ok(());
            }
            let mut file = std::fs::File::create(".gitignore")?;
            file.write_all(content.unwrap().as_bytes())?;
        }
    }

    // Ensure cursor is shown before exiting
    reset_terminal();

    Ok(())
}

async fn get_gitignore_content(append: bool) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        reset_terminal();
        std::process::exit(0);
    })?;

    let response = reqwest::get(format!("{}/list", API_URL)).await?;
    let mut keywords: Vec<String> = response
        .text()
        .await?
        .replace("\n", ",")
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    if append {
        keywords.insert(0, "Append to .gitignore".to_string());
    } else {
        keywords.insert(0, "Generate .gitignore".to_string());
    }

    let mut chosen_keywords = HashSet::new();

    println!("Please type the languages, editors, operating systems and IDEs you will use. Enter to generate.");

    loop {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        let input = match FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("> ")
            .items(&keywords)
            .default(0)
            .max_length(8) // Show only the top 8 options
            .interact_opt()?
        {
            Some(index) => index,
            None => {
                // Handle Ctrl-D (EOF)
                println!("EOF detected. Exiting...");
                break;
            }
        };

        if input == 0 {
            let confirm = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Confirm generation?")
                .default(0)
                .item("Yes")
                .item("No")
                .interact()?;

            if confirm == 0 {
                break;
            } else {
                println!("Exiting...");
                return Ok(None);
            }
        } else {
            chosen_keywords.insert(keywords[input].clone());
        }
    }

    if chosen_keywords.is_empty() {
        println!("No languages or operating systems selected. Exiting...");
        return Ok(None);
    }

    println!("Generating...");
    let response = reqwest::get(format!(
        "{}/{}",
        API_URL,
        chosen_keywords
            .iter()
            .cloned()
            .collect::<Vec<String>>()
            .join(",")
    ))
    .await?;
    let content = response.text().await?;

    Ok(Some(content))
}

fn reset_terminal() {
    // Ensure cursor is shown before exiting
    let mut stdout = std::io::stdout();
    write!(stdout, "{}", Show).unwrap();
    stdout.flush().unwrap();
}
