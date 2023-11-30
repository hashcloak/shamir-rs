mod shamir_secret_sharing;

use shamir_secret_sharing::{get_shares_secret, generate_secret};
use shamir_secret_sharing::Fq;
use std::env;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

const N: u64 = 3;
const T: usize = 1;

// Send message to other port on localhost
async fn send_to_port(port: u16, message: String) {
    if let Ok(mut stream) = TcpStream::connect(format!("127.0.0.1:{}", port)).await {
        let _ = stream.write_all(message.as_bytes()).await;
    }
}

// Handeling of incoming commands for MPC party
async fn mpc_party(mut incoming_connection: TcpStream, incoming_port: u16, secret: Fq, port: String, shares: Arc<Mutex<Vec<(u16, u64)>>>) {
    let mut buf = [0; 1024];
    let n = incoming_connection.read(&mut buf).await.unwrap();
    let msg = String::from_utf8_lossy(&buf[..n]);
    let command = msg.trim();

    match command.split_whitespace().collect::<Vec<&str>>().as_slice() {
        // Trigger to communicate shares to other parties
        ["COMMUNICATE_SHARES", target_ports @ ..] => {
            // Generate n shares, keep 1 and send n-1 to other parties
            println!("Secret is {}", secret);
            let shares = get_shares_secret(secret, N, T);
            println!("Shares are {:#?}", shares);
            for (index, &target_port) in target_ports.iter().enumerate() {
                if let Ok(target_port) = target_port.parse::<u16>() {
                    send_to_port(target_port, format!("RECEIVE_SHARE {}", shares[index])).await;
                }
            }
        },
        // Handle an incoming share of another party
        ["RECEIVE_SHARE", value] => {
            println!("Received on port {}: SHARE {}", port, value);
            let received_value: u64 = value.parse().unwrap();
            // TODO: incoming port has some sort of mapping (for example expected 8081, 8082 but gives 53249, 53273)
            shares.lock().await.push((incoming_port, received_value));
        },
        // Print the shares this server holds - For testing purposes
        ["SHOW_SHARES"] => {
            let shares_lock = shares.lock().await;
            println!("Shares on port {}: {:?}", port, *shares_lock);
        },
        _ => println!("Invalid command"),
    }
}

#[tokio::main]
async fn main() {
    // Get portnumber from commandline
    let args: Vec<String> = env::args().collect();
    let port = args.get(1).expect("Port number is required").clone();

    // Obtain a random secret
    let secret = generate_secret();

    // This is where received shares from other parties are stored
    let shares = Arc::new(Mutex::new(Vec::<(u16, u64)>::new()));

    // Start listening for incoming connection requests on localhost with given port
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await.unwrap();
    println!("Server running on port {}", port);

    loop {
        // Waits for incoming connection attempt, stores established connection to the client
        let (incoming_connection, incoming_address) = listener.accept().await.unwrap();
        // Retrieve the incoming port
        // TODO this gives values like 59671, 59675 instead of 8081 and 8082
        let incoming_port = incoming_address.port();
        println!("Client port: {}", incoming_port);

        let port_clone = port.clone();
        let shares_clone = shares.clone();
        // Create and start new async task, which is the MPC party
        tokio::spawn(async move {
            mpc_party(incoming_connection, incoming_port, secret, port_clone, shares_clone).await;
        });
    }
}
