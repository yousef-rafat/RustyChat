use std::{io::ErrorKind, net::{SocketAddr, TcpListener}};
use tokio::{io::AsyncBufReadExt, net::TcpListener as TokioTcpListener}; 
use rand::Rng;
use tokio::io::AsyncWriteExt;

pub async fn init_server(choosen_port: Option<u16>) {

    let port;

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

    let listener= TokioTcpListener::bind(port_addr).await.expect("Couldn't Create Server");

    println!("Your chat server is at: 127.0.0.1:{}", port);

    process(listener).await;

}

async fn process(listener: TokioTcpListener) {
    loop {

        let (socket, _addr) = match listener.accept().await {
            Ok(connection) => connection,
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        };

        tokio::spawn(async move {
            
            let mut buffer = String::new(); // vec![0u8; 1024];
            let mut reader = tokio::io::BufReader::new(socket);

            loop {

                match reader.read_line(&mut buffer).await {//socket.read(&mut buffer).await {

                    Ok(0) => {
                        println!("Connection Closed!");
                        std::process::exit(0);
                    }

                    Ok(_) => {

                        println!("Received: {:?}", buffer.trim_end());

                        if let Err(e) = reader.get_mut().write_all(buffer.as_bytes()).await {
                            eprintln!("Error writing to socket: {e}");
                            break;
                        }

                        buffer.clear();
                    },

                    Err(e) => {
                        eprintln!("Error reading from socket: {}", e);
                        std::process::exit(0);
                    }
                };     
            }
        });    
    }
}

fn is_port_used(port: u16) -> bool {
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

    let mut port;
    port = rand::thread_rng().gen_range(1024..65535);

    while is_port_used(port) {
        port = rand::thread_rng().gen_range(1024..65535);
    }
    
    port
}
