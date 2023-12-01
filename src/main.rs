mod shamir_secret_sharing;

use shamir_secret_sharing::Fq;
use shamir_secret_sharing::{generate_secret, get_shares_secret};
use std::collections::HashMap;
use std::sync::Arc;
use std::{clone, env};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::{self, Duration};

const N: u64 = 3;
const T: usize = 1;

// Send message to other port on localhost
async fn send_to_port(port: u16, message: String) {
    if let Ok(mut stream) = TcpStream::connect(format!("127.0.0.1:{}", port)).await {
        let _ = stream.write_all(message.as_bytes()).await;
    }
}

fn get_shares_for_parties(secret: Fq, current_id: u64, other_parties: [u64; 2]) -> Vec<(Fq, Fq)> {
    let mut inputs = Vec::new();
    inputs.push(current_id);
    inputs.push(other_parties[0]);
    inputs.push(other_parties[1]);
    get_shares_secret(secret, inputs, T)
}

// Handeling of incoming commands for MPC party
async fn mpc_party(
    mut incoming_connection: TcpStream,
    out_connections: Arc<Mutex<HashMap<String, TcpStream>>>,
    secret: Fq,
    current_id: u64,
    other_parties: [u64; 2],
    shares_storage: Arc<Mutex<Vec<(u64, Fq, Fq)>>>,
) {
    let mut buf = [0; 1024];
    let n = incoming_connection.read(&mut buf).await.unwrap();
    let msg = String::from_utf8_lossy(&buf[..n]);
    let command = msg.trim();

    match command.split_whitespace().collect::<Vec<&str>>().as_slice() {
        // Trigger to communicate shares to other parties
        ["COMMUNICATE_SHARES", target_ports @ ..] => {
            // Generate n shares, keep 1 and send n-1 to other parties
            println!("Secret is {}", secret);
            let mut shares = get_shares_for_parties(secret, current_id, other_parties);

            println!("Shares are {:#?}", shares);
            // Send shares to other ports
            for (index, &target_port) in target_ports.iter().enumerate() {
                if let Ok(target_port) = target_port.parse::<u16>() {
                    // TODO get the right share for the right port.
                    let share = shares.remove(index);
                    println!(
                        "sending RECEIVE_SHARE {} {} to port {}",
                        share.0, share.1, target_port
                    );

                    let mut connections_lock = out_connections.lock().await;
                    println!("The hashmap is {:?}", connections_lock);
                    let out_stream = connections_lock.get_mut(&target_port.to_string()).unwrap();

                    out_stream.writable().await.unwrap();
                    println!("Remote address: {:?}", out_stream.peer_addr());
                    let result_writing = out_stream
                        .write(
                            format!("RECEIVE_SHARE {} {} {}", current_id, share.0, share.1)
                                .as_bytes(),
                        )
                        .await;

                    match result_writing {
                        Ok(size) => {
                            println!("Size of sent: {}", size);
                        }
                        Err(msg) => {
                            println!("Error writing to stream: {}", msg);
                        }
                    }

                    // send_to_port(target_port, format!("RECEIVE_SHARE {} {} {}", current_id, share.0, share.1)).await;
                }
            }
            // Store own share, which is the last share that's left over
            let own_share = shares.remove(0);
            // Wrong port id, but we're going to substitute this all for id's anyway
            shares_storage
                .lock()
                .await
                .push((current_id, own_share.0, own_share.1));
        }
        // Handle an incoming share of another party
        ["RECEIVE_SHARE", id_party, x, y] => {
            println!(
                "Received at party {} from party {}: SHARE {} {}",
                current_id.to_string(),
                id_party,
                x,
                y
            );
            let received_id_party: u64 = id_party.parse().unwrap();
            let received_x: Fq = x.parse().unwrap();
            let received_y: Fq = y.parse().unwrap();
            shares_storage
                .lock()
                .await
                .push((received_id_party, received_x, received_y));
        }
        // Print the shares this server holds - For testing purposes
        ["SHOW_SHARES"] => {
            let shares_lock = shares_storage.lock().await;
            println!(
                "Shares of party {}: {:?}",
                current_id.to_string(),
                *shares_lock
            );
        }
        _ => println!(""),
    }
}

// Async function to repeatedly try connecting to port 8081
async fn connect_to_ports(
    targets: Vec<String>,
    local_port: String,
    connections: Arc<Mutex<HashMap<String, TcpStream>>>,
) {
    for target_port in targets {
        loop {
            match TcpStream::connect(format!("127.0.0.1:{}", target_port)).await {
                Ok(mut stream) => {
                    stream
                        .write(format!("HELLO {}", local_port).as_bytes())
                        .await
                        .unwrap();
                    connections.lock().await.insert(target_port, stream);
                    break;
                }
                Err(e) => {
                    // eprintln!("Failed to connect to port {}: {}", target_port, e);
                    time::sleep(Duration::from_secs(3)).await;
                }
            }
            time::sleep(Duration::from_secs(1)).await; // Wait before retrying
        }
    }
}

fn parse_party(party_str: &str) -> (u64, u16) {
    let parts: Vec<&str> = party_str.split(':').collect();
    if parts.len() != 2 {
        eprintln!("Invalid party format. Expected format: id:port");
        std::process::exit(1);
    }
    let id = parts[0].parse::<u64>().expect("Invalid id");
    let port = parts[1].parse::<u16>().expect("Invalid port number");
    (id, port)
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    // Input: current_id "id1:port1" "id2:port2" "id3:port3"
    if args.len() != 5 {
        eprintln!("Usage: program current_id \"party1_id:party1_port\" \"party2_id:party2_port\" \"party3_id:party3_port\"");
        std::process::exit(1);
    }

    // Parse all the arguments
    let current_id = &args[1].parse::<u64>().expect("Invalid current id");
    let party1 = parse_party(&args[2]);
    let party2 = parse_party(&args[3]);
    let party3 = parse_party(&args[4]);
    let parties = vec![party1, party2, party3];
    let port = parties
        .iter()
        .find(|(id, _)| id == current_id)
        .map(|(_, port)| *port)
        .expect("Current ID does not match any party");
    let other_parties: Vec<(u64, u16)> = parties
        .into_iter()
        .filter(|(id, _)| id != current_id)
        .collect();

    // Obtain a random secret
    let secret = generate_secret();

    // This is where received shares from other parties are stored
    let shares = Arc::new(Mutex::new(Vec::<(u64, Fq, Fq)>::new()));

    // Start listening for incoming connection requests on localhost with given port
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();
    println!("Server running on port {}", port);

    let other_port_1 = other_parties[0];
    let other_port_2 = other_parties[1];
    let current_party_id = (*current_id).clone();

    // Tries to connect to the other clients
    let connections = Arc::new(Mutex::new(HashMap::new()));
    let connections_clone = Arc::clone(&connections);
    tokio::spawn(connect_to_ports(
        vec![other_port_1.1.to_string(), other_port_2.1.to_string()],
        port.clone().to_string(),
        connections_clone,
    ))
    .await
    .unwrap();

    // Print the connections after connecting with parties
    println!("Established connections: {:?}", connections);

    let clonned_conn_iter = Arc::clone(&connections);

    loop {
        // Waits for incoming connection attempt, stores established connection to the client
        let (incoming_connection, _) = listener.accept().await.unwrap();

        let shares_clone = shares.clone();
        let out_connections = Arc::clone(&connections);
        // Create and start new async task, which is the MPC party
        tokio::spawn(async move {
            mpc_party(
                incoming_connection,
                out_connections,
                secret,
                current_party_id,
                [other_port_1.0, other_port_2.0],
                shares_clone,
            )
            .await;
        });
    }
}
