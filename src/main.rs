use clap::Parser;
use std::collections::HashMap;
use colored::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help("Kaydetmek istediğiniz içeriği yapıştırın."))]
    content: Option<String>,
}

#[derive(Debug, serde_derive::Deserialize)]
struct Paste {
    hash: String,
}

#[derive(Debug, serde_derive::Deserialize)]
struct ClientError {
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args: Args = Args::parse();
    let mut post_data = HashMap::new();

    if args.content.is_none() {
        println!("Error: {}", "Lütfen içerik girin. Yardım için -h bayrağını ekleyin.".red().underline().bold());
        return Ok(());
    }
    
    post_data.insert("code", args.content);

    let response = reqwest::Client::new()
        .post("https://paste.ahmetbarut.net/api/paste")
        .header("Accept", "application/json")
        .json(&post_data)
        .send()
        .await?;

    let is_client_error = response.status().is_client_error();
    if is_client_error {
        let error_message: ClientError = response.json().await?;
        println!("Error: {}", error_message.message);
        return Ok(());
    }

    let paste: Paste = response.json().await?;

    let paste_url = format!("https://paste.ahmetbarut.net/{}", paste.hash);

    print!("");
    println!("Paste : {}", paste_url.green().bold().underline());

    Ok(())
}
