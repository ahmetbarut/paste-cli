use clap::Parser;
use colored::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use rusqlite::{params, Connection, Result};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, help("Kaydetmek istediğiniz içeriği yapıştırın."))]
    content: Option<String>,
    #[arg(short, help("Daha önce kaydettiğiniz içerikleri görüntüleyin"))]
    my_pastes: bool,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args: Args = Args::parse();
    let mut post_data = HashMap::new();

    if args.my_pastes {
        my_pastes();
        return Ok(());
    }

    if args.content.is_none() {
        println!(
            "Error: {}",
            "Lütfen içerik girin. Yardım için -h bayrağını ekleyin."
                .red()
                .underline()
                .bold()
        );
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

    store_local_paste(paste.hash.to_string());
    let paste_url = format!("https://paste.ahmetbarut.net/{}", paste.hash);

    print!("");
    println!("Paste : {}", paste_url.green().bold().underline());

    Ok(())
}

fn my_pastes() {
    let conn = get_connection().unwrap();

    let mut stmt = conn.prepare("SELECT hash, created_at FROM pastes").unwrap();

    let paste_iter = stmt.query_map([], |row| {
        Ok(LocalPaste {
            hash: row.get(0)?,
            created_at: row.get(1)?,
        })
    }).unwrap();

    for paste in paste_iter {
        let p: LocalPaste = paste.unwrap();
        println!("+{:-^50}+", "");
        println!("| URL: {}{}{} |", "\x1b[32m", format!("https://paste.ahmetbarut.net/{}", p.hash), "\x1b[0m");
        println!("| Created At: {:^32} |", p.created_at);
        println!("+{:-^50}+", "");
    }
}

fn store_local_paste(paste_id: String) -> String{
    let conn = get_connection().unwrap();
    let time = chrono::Local::now().to_string();
    conn.execute(
        "INSERT INTO pastes (hash, created_at) VALUES (?1, ?2)",
        params![paste_id, time],
    ).unwrap();

 
    return paste_id.to_string();
}

#[derive(Debug, serde_derive::Deserialize)]
pub struct Paste {
    pub hash: String,
}

#[derive(Debug, serde_derive::Deserialize)]
pub struct ClientError {
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalPaste {
    pub hash: String,
    pub created_at: String,
}

fn get_connection() -> Result<Connection> {
    let conn = Connection::open("pastes.sqlite")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pastes (
            id INTEGER PRIMARY KEY,
            hash TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}