use dialoguer::{theme::ColorfulTheme, FuzzySelect, Select};
use reqwest;
use std::collections::HashSet;
use std::io::Write;
use termion::cursor::Show; // Add this import
use termion::raw::IntoRawMode;
use tokio; // Add this import

const API_URL: &str = "https://www.toptal.com/developers/gitignore/api";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get(format!("{}/list", API_URL)).await?;
    let mut keywords: Vec<String> = response
        .text()
        .await?
        .replace("\n", ",")
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    keywords.insert(0, "Generate .gitignore".to_string());
    println!("{:?}", keywords);

    let mut chosen_keywords = HashSet::new();

    println!("Please type the languages and operating systems you will use. Enter to generate.");

    loop {
        let input = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("> ")
            .items(&keywords)
            .default(0)
            .max_length(8) // Show only the top 8 options
            .interact()?;

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
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock().into_raw_mode()?;
    write!(stdout, "{}", Show)?;

    Ok(())
}
