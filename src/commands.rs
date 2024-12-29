use tokio::fs::{ File, metadata };
use tokio::io::AsyncWriteExt;
use std::error::Error;
use regex::Regex;
use std::collections::HashSet;

pub fn filter_duplicates(chat_history: Vec<String>) -> Vec<String> {
    let mut unique_messages = HashSet::new(); // To track unique messages
    let mut filtered_history = Vec::new();   // To store filtered messages

    for message in chat_history {
        // Insert message into the HashSet; if it's not a duplicate, add it to the filtered history
        if unique_messages.insert(message.clone()) {
            filtered_history.push(message);
        }
    }

    filtered_history
}

pub async fn save_chat(username: String, message_content: Vec<String>) -> Result<(), Box<dyn Error>> {
    let chat_content = message_content.join("\n");
    let mut file_name = format!("{}.txt", username);

    let mut counter = 1;
    while metadata(&file_name).await.is_ok() {
        file_name = format!("{}_{}.txt", username, counter);
        counter += 1;
    }

    let mut file = File::create(file_name).await?;
    file.write_all(chat_content.as_bytes()).await?;

    Ok(())
}

pub async fn clear_terminals<T>(reader: &mut tokio::io::BufReader<T>) 
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let clear_code = "\x1b[2J\x1b[H"; // ANSI escape sequence to clear the terminal

    if let Err(e) = reader.get_mut().write_all(clear_code.as_bytes()).await {
        eprintln!("Failed to clear terminal: {e}");
    }
}

pub async fn show_users<T>(reader: &mut tokio::io::BufReader<T>, users: Vec<String>)
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let mut user_list = String::from("Current users in the chat:\n\r");
    for user in users {
        user_list.push_str(&format!("- {}\n\r", user));
    }

    if let Err(e) = reader.get_mut().write_all(user_list.as_bytes()).await {
        eprintln!("Failed to send user list: {e}");
    }   
}

pub async fn check_for_magic_commands(text: &String) -> Option<&str> {

    let magic_command_pattern = r"&[a-zA-Z_][a-zA-Z0-9_]*";
    let re = Regex::new(magic_command_pattern).unwrap();

    if let Some(matched_command) = re.find(&text) {
        match matched_command.as_str() {
            "&save_text" => return Some("&save_text"),
            "&clear_screen" => return Some("&clear_screen"),
            "&show_users" => return Some("&show_users"),
            _ => {
                eprintln!("Unkown Magic Command!");
                return Some("None");
            }
        } 
    } else {
        return Some("None");
    }
}
