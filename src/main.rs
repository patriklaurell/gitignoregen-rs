use ctrlc; // Add this import
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Select};
use reqwest;
use std::collections::HashSet;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use termion::cursor::Show; // Add this import
use tokio; // Add this import

const API_URL: &str = "https://www.toptal.com/developers/gitignore/api";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    keywords.insert(0, "Generate .gitignore".to_string());

    let mut chosen_keywords = HashSet::new();

    println!("Please type the languages and operating systems you will use. Enter to generate.");

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
                return Ok(());
            }
        } else {
            chosen_keywords.insert(keywords[input].clone());
        }
    }

    if chosen_keywords.is_empty() {
        println!("No languages or operating systems selected. Exiting...");
        return Ok(());
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

    let mut file = std::fs::File::create(".gitignore")?;
    file.write_all(content.as_bytes())?;

    // Ensure cursor is shown before exiting
    reset_terminal();

    Ok(())
}

fn reset_terminal() {
    // Ensure cursor is shown before exiting
    let mut stdout = std::io::stdout();
    write!(stdout, "{}", Show).unwrap();
    stdout.flush().unwrap();
}
