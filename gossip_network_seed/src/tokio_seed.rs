/// Contains the Struct and functions for Seed Node

// Including the packages.
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

// constants for specific requests.
const JOIN_REQUEST_MESSAGE: &str = "JOIN_REQUEST";
const GET_CONNECTED_NODES_REQUEST: &str = "GET_CONNECTED_NODES_REQUEST";
const DEAD_NODE_MESSAGE: &str = "DEAD_NODE";

// Represents a Seed
pub struct Seed {
    seed_no: i32, // seed Identifier
    connected_networks: HashSet<String>, // IPs & Port of Unique peers connected to these seed.
}

impl Seed {
    pub fn new(seed_no: i32) -> Self {
        Seed {
            seed_no,
            connected_networks: HashSet::new(),
        }
    }
    // handles any incoming requests and responds.
    async fn handle_connection(seed: Arc<Mutex<Seed>>, mut stream: TcpStream) {
        // Read from the buffer and parse the request.
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await.expect("Failed to read from stream");
        let received_message = String::from_utf8_lossy(&buffer[..n]).to_string();
        let mut message_list: Vec<&str> = received_message.split('|').collect();
        message_list.iter_mut().for_each(|s| *s = s.trim());

        // Obtain the lock on seed.
        let mut seed_guard = seed.lock().await;
        // Handle JOIN REQUEST from peer.
        if message_list[0] == JOIN_REQUEST_MESSAGE {
            let peer_addr = message_list[1];
            // add the peer to the list of connected nodes
            seed_guard.connected_networks.insert(peer_addr.to_string());
            println!("Seed #{}: Received JOIN request from {}.", seed_guard.seed_no, peer_addr);
            // respond to the peer
            let response = format!("Successfully Connected to {:?}", peer_addr);
            stream.write_all(response.as_bytes()).await.expect("Failed to write response");
        }
        // hadnles GET_CONNECTED_NODES_REQUEST
        else if message_list[0] == GET_CONNECTED_NODES_REQUEST {
            let peer_addr = message_list[1];
            // Extract the list of distinct connected nodes other than the requesting peer.
            let connected_nodes_list: Vec<String> = seed_guard.connected_networks
                .iter().cloned()
                .filter(|node| node!= peer_addr) 
                .collect();
            // responsd to the peer
            let response = format!("Connected Nodes: {:?}", connected_nodes_list);
            stream.write_all(response.as_bytes()).await.expect("Failed to write response");
        }
        // Handles DEAD_NODE_MESSAGE
        else if message_list[0] == DEAD_NODE_MESSAGE {
            let dead_node = message_list[1];
            let reporting_node = message_list[3];
            // Print the received dead node request and the reporting node.
            println!("Seed #{}: Dead node:[{}] , reported by:[{}]",
                    seed_guard.seed_no, dead_node, reporting_node);
            // remove the dead node from the list of connections.
            if seed_guard.connected_networks.remove(dead_node) {
                println!("Seed #{}: Successfully Removed Dead node:[{}]",
                    seed_guard.seed_no, dead_node);
            } else {
                println!("Seed #{}: Node [{}] not found in the list of connected networks.",
                    seed_guard.seed_no, dead_node);
            }
        } else {
            println!("Unexpected Message\n");
        }
    }

    // Starts a listener for each Seed.
    pub async fn start_listener(seed: Arc<Mutex<Seed>>, ip: String, port: String) {
        let addr = SocketAddr::new(ip.parse().unwrap(), port.parse().unwrap());
        let listener = TcpListener::bind(&addr).await.expect("Failed to bind listener");
        println!("Seed #{}: listening on {}:{}", seed.lock().await.seed_no, ip, port);
        // loop for handling any incoming connections.
        loop {
            let (stream, _) = listener.accept().await.expect("Failed to accept connection");
            let seed_clone = seed.clone();
            tokio::spawn(async move {
                Seed::handle_connection(seed_clone, stream).await;
            });
        }
    }
}
