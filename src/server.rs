use std::{io::ErrorKind, net::{SocketAddr, TcpListener}};
use tokio::{io::AsyncBufReadExt, net::TcpListener as TokioTcpListener, sync::broadcast}; 
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use rand::Rng;
use crate::commands;

// type for creating our maps
type UserMap = Arc<Mutex<HashMap<String, String>>>;

pub async fn init_server(choosen_port: Option<u16>) {

    let port;

    // check if the user choosen port is used or not
    // if used assign a different one
    match choosen_port {
        Some(p) => { 
            if is_port_used(p) { 
                eprintln!("Port is used!\nChoosing a random port instead");
                port = get_port();
            } else {
                port = p; 
            }
        },
        None => { port = get_port(); }
    }

    let port_addr: String = format!("127.0.0.1:{}", port);

    // create a TCP server for our chatting
    let listener= TokioTcpListener::bind(port_addr).await.expect("Couldn't Create Server");

    println!("Your chat server is at: 127.0.0.1:{}", port);

    process(listener).await;

}

async fn process(listener: TokioTcpListener) {

    // create the broadcast channel that our users will use to communicate on
    // every sending message will go to everyone one on the this channel
    let (tx,  _rx) = broadcast::channel(10);
    // user map for mapping the usernames with their addresses
    let user_map: UserMap = Arc::new(Mutex::new(HashMap::new()));

    loop {

        let (socket, addr) = match listener.accept().await {
            Ok(connection) => connection,
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        };

        // cloning to avoid rust errors
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        let user_map = Arc::clone(&user_map);

        tokio::spawn(async move {

            // buffer for storing our text
            // username_buffer for storing usernames
            let mut buffer = String::new();
            let mut username_buffer = String::new();

            let mut reader = tokio::io::BufReader::new(socket);

            loop {

                // ask each user for their usernames
                if let Err(e) = reader.get_mut().write_all("\rEnter Username: ".as_bytes()).await {
                    eprintln!("ERROR in writing text: {e}");
                    return;
                }

                // read the user's name
                if let Err(e) = reader.read_line(&mut username_buffer).await {
                    eprintln!("ERROR in reading username: {e}");
                    return;
                }

                let username = username_buffer.trim().to_string();

                if username.is_empty() { // refuse to process empty
                    eprintln!("Can't process an empty username!");
                    continue;
                }

                let username_taken;

                // lock the map and check if the username is taken
                {
                    let mut map = user_map.lock().await;
                    username_taken = map.values().any(|v| v == &username); 
                    if !username_taken {
                        map.insert(addr.clone().to_string(), username.clone()); // insert the new user
                    }
                }
        
                // if username is taken, send the appropriate response
                if username_taken {
                    if let Err(e) = reader.get_mut().write_all(b"Username is already taken. Try again.\n").await {
                        eprintln!("Error username is already taken: {e}");
                    }
                    username_buffer.clear(); // Clear buffer for retry
                } else {
                   // print welcome to the new user that has joined the channel
                   let welcome_message = format!("{} has joined the chat.\n\r", {username.clone()});

                   if let Err(e) = tx.send((welcome_message.clone(), addr)) {
                    eprintln!("Error broadcasting welcome message: {e}");

                    if let Err(e) = reader.get_mut().write_all("Joined the channel.\n\r".as_bytes()).await {
                        eprintln!("Error sending welcome message to new user: {e}");
                    }
                }

                    break;
                }
            }
            // will save the conversation in the chat
            let mut message_content: Vec<String> = vec![];

            loop {

                // searches the map we created above to get the username from the address
                let username = {
                    let map = user_map.lock().await;
                    match map.get(&addr.to_string()) {
                        Some(user) => user.clone(),
                        None => {
                            eprintln!("Problem with finding username");
                            continue; // Skip to the next iteration of the loop
                        }
                    }
                };

                tokio::select! {
                    // Handle reading from the client
                    read_result = reader.read_line(&mut buffer) => {
                        match read_result {

                            Ok(0) => {
                                println!("Connection Closed!");
                                break;
                            }

                            Ok(_) => {

                                let input = buffer.to_string();

                                let processed_input = process_backspaces(&input);
                                
                                // for debugging
                                println!("Received: {:?}", processed_input.trim_end());
                                                            
                                // Send the messages to all users
                                if !processed_input.is_empty() {

                                    let message = format!("{}: {}", username, processed_input.clone());

                                    message_content.push(message.clone());

                                    // checks if any magic command in the code and match it with the correct function
                                    match commands::check_for_magic_commands(&processed_input).await {
                                        Some(command) => {
                                            match command {
                                                "&save_text" => {
                                                    // filter the duplicates and save the chat
                                                    // later notify the user that the chat was saved
                                                    let filted_messages = commands::filter_duplicates(message_content.clone());

                                                    match commands::save_chat(username.clone(), filted_messages.clone()).await {
                                                        Ok(_) => {},
                                                        Err(e) => {eprintln!("Problem with saving chat: {e}")}
                                                    };

                                                    if let Err(e) = reader.get_mut().write_all("The text chat was saved!.\n\r".as_bytes()).await {
                                                        eprintln!("Error sending success saving-chat message to user: {e}");
                                                    }
                                                }
                                                "&clear_screen" => {
                                                    commands::clear_terminals(&mut reader).await;
                                                }
                                                "&show_users" => {
                                                    let map = user_map.lock().await; 

                                                    // Extract the usernames (values from the map)
                                                    let usernames: Vec<String> = map.values().cloned().collect();
                                                    commands::show_users(&mut reader, usernames.clone()).await;
                                                }
                                                "&help" => {
                                                    commands::display_help(&mut reader).await;
                                                }
                                                _ => {
                                                    // if no magic command, send the message normally
                                                    if let Err(e) = tx.send((message.clone(), addr)) {
                                                        eprintln!("Error while sending a message: {e}");
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        None => {}
                                    }
                                }
                                
                                // clearing the buffer is important for correct message transimition
                                buffer.clear();
                            }

                            Err(e) => {
                                eprintln!("Error reading from socket: {}", e);
                                break;
                            }
                        }
                    }

                    // Handle broadcasting messages
                    recv_result = rx.recv() => {

                        match recv_result {

                            Ok((msg, other_addr)) => {
                                let processed_msg = process_backspaces(&msg);
                                
                                // pushing the messages to a buffer so  we can save it if the user want to
                                message_content.push(processed_msg.clone());
                    
                                if addr != other_addr {
                                    if let Err(e) = reader.get_mut().write_all(processed_msg.as_bytes()).await {
                                        eprintln!("Error writing to socket: {e}");
                                        break;
                                    }
                                }
                            }

                            Err(e) => {
                                eprintln!("Error receiving messages: {e}");
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
}


fn process_backspaces(buffer: &str) -> String {
    // function used to remove the backspace symbol: '\u{8}' from text
    let mut result = String::new();
    
    for c in buffer.chars() {
        if c == '\u{8}' {
            result.pop();
        } else {
            result.push(c);
        }
    }

    result
}

fn is_port_used(port: u16) -> bool {
    // Checks if the port is used or not by setting a tcp connection and dropping it after knowing it's not in use
    let addr = format!("127.0.0.1:{}", port);
    let socket_addr: SocketAddr = addr.parse().expect("Invalid address format");

    match TcpListener::bind(&socket_addr) {
        Ok(listner) => {
            drop(listner);
            false 
        },
        Err(e) => match  e.kind() {
            ErrorKind::AddrInUse => true,
            _ => false
        }
    }
}

fn get_port() -> u16 {
    // get a randomly assigned port
    let mut port;
    port = rand::thread_rng().gen_range(1024..65535);

    while is_port_used(port) {
        port = rand::thread_rng().gen_range(1024..65535);
    }
    
    port
}