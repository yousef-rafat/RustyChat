use std::env;
mod server;

const HELP: &str = "RustyChat is a simple local chat server, allowing for no latency communication with users on the same network";

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    handle_args(&args).await;
}

async fn handle_args(args: &Vec<String>) {

    if args.len() < 2 {
        eprintln!("Usage: RustyChat <command> [options]");
        return;
    }

    match args[1].as_str() {

        "help" => { println!("{HELP}"); },

        "init" => { 
            println!("Initializing the chat server!");

            let port = if args.len() > 2 {
                match args[2].parse::<u16>() {
                    Ok(p) => Some(p),
                    Err(_) => {
                        eprintln!("Invalid port number, defaulting to 8080.");
                        Some(8080)
                    }
                }
            } 
            else { 
                println!("No port was choosen. Defaulting to 8080.");
                Some(8080)
            };

            server::init_server(port).await;
        },

        _ => {
            eprintln!("Error, unknown command: {}", args[1]);
        }
    }
}