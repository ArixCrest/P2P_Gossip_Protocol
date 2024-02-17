/// Contains the Struct and functions for Peer Node

// importing necessary packages
use std::collections::HashSet;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use chrono::prelude::*;

// Constants for specific reply
const JOIN_REQUEST_MESSAGE: &str = "JOIN_REQUEST";
const GET_CONNECTED_NODES_REQUEST: &str = "GET_CONNECTED_NODES_REQUEST";
const DEAD_NODE_MESSAGE: &str = "DEAD_NODE";

// Peer Struct
pub struct Peer {
    pub peer_no: i32, // Peer identifier
    pub local_addr: String, // stores peer address i.e IP:PORT
    pub seed_nodes: Vec<String>, // Stores the connected seed nodes.
    pub connected_nodes: HashSet<String>, // stores the connected distinct peer nodes
    pub message_list: HashSet<String>, // stores the messages received
    pub creation_time: DateTime<Utc>, // stores the local time when this peer was created.
}

impl Peer {
    pub fn new(peer_no: i32, local_addr:String, seed_nodes: Vec<String>) -> Self {
        let creation_time = Utc::now();
        Peer {
            peer_no,
            local_addr,
            seed_nodes,
            connected_nodes: HashSet::new(),
            message_list: HashSet::new(),
            creation_time,
        }
    }
    // Sends a Request to the seed nodes to join.
    pub async fn join_seed_nodes(&mut self,) {
        for seed_node in &self.seed_nodes {
            match TcpStream::connect(seed_node).await {
                Ok(mut stream) => {
                    let response = format!("{}|{}|{}", JOIN_REQUEST_MESSAGE, &self.local_addr, self.elapsed_time());
                    stream.write_all(response.as_bytes()).await.unwrap();
                    // println!("Sent JOIN_REQUEST to seed: {:?}", seed_node);

                    // Wait for the response
                    let mut buffer = [0; 1024];
                    if let Ok(n) = stream.read(&mut buffer).await {
                        let _response = String::from_utf8_lossy(&buffer[..n]);
                        // println!("Received response from seed: {:?}", response);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect to {:?} with local address {:?}: {}", seed_node, self.local_addr, e);
                }
            }
        }
    }
    // Internal function the parse the message.
    fn _extract_nodes(input_string: &str) -> Vec<String> {
        let nodes_str = input_string.trim().replace("Connected Nodes: [", "");
        let nodes_str = &nodes_str.replace("]", "");
        let nodes_str = &nodes_str.replace("\\", "");
        let nodes_str = &nodes_str.replace("\"", "");

        let nodes_str: Vec<String> = nodes_str.split(", ").map(|s| s.to_string()).collect();
    
        // Gather parsed nodes into a Vec and return it.
        return nodes_str;
    }
    
    // Queries the connected nodes form each seed.
    pub async fn query_connected_nodes(&mut self) {
        for seed_node in &self.seed_nodes {
            if let Ok(mut stream) = TcpStream::connect(seed_node).await {
                // Send GET_CONNECTED_NODES_REQUEST message to seed
                let response = format!("{}|{}|{}", GET_CONNECTED_NODES_REQUEST, &self.local_addr, self.elapsed_time());
                stream.write_all(response.as_bytes()).await.unwrap();
                // println!("Sent GET_CONNECTED_NODES_REQUEST to seed: {:?}", seed_node);

                // Wait for the response
                let mut buffer = [0; 1024];
                if let Ok(n) = stream.read(&mut buffer).await {
                    let response = String::from_utf8_lossy(&buffer[..n]);
                    // gets the ips c onnected to the seed other than the current peer.
                    let connected_nodes: Vec<String> = Peer::_extract_nodes(&response);

                    // Update the connected nodes list if we get other peer IPs.
                    if connected_nodes[0].len() > 0{
                        self.connected_nodes.extend(connected_nodes);
                    }
                }

            } else {
                eprintln!("Peer@{}: Failed to connect to {:?}", self.local_addr, seed_node);
            }
        }
    }

    // Sends a DEAD_NODE message to the seed nodes
    pub async fn declare_node_dead(&mut self, dead_node : String){
        for seed_node in &self.seed_nodes {
            match TcpStream::connect(seed_node).await {
                Ok(mut stream) => {
                    // Send GET_CONNECTED_NODES_REQUEST message to seed
                    let response = format!("{}|{}|{}|{}",
                        DEAD_NODE_MESSAGE, dead_node, &self.elapsed_time(), &self.local_addr);
                    if let Err(err) = stream.write_all(response.as_bytes()).await {
                        eprintln!("Failed to send DEAD_NODE request to seed {:?}: {}", seed_node, err);
                    }else{
                        println!("Peer@{}: {}", self.local_addr, response);
                    }
                },
                Err(err) => {
                    eprintln!("Failed to connect to {:?}: {}", seed_node, err);
                }
            }
        }
    }

    // Returns the elapsed time since the creation time this acts as the local timestamp.
    pub fn elapsed_time(&self) -> String {
        let current_time = Utc::now();
        let duration = current_time.signed_duration_since(self.creation_time);

        let _hours = duration.num_hours();
        let minutes = duration.num_minutes() % 60;
        let seconds = duration.num_seconds() % 60;
        let milliseconds = (duration.num_milliseconds() % 1000).abs();

        // Format hours, minutes, seconds, and milliseconds using num-format
        format!("{:02}:{:02}:{:03}",minutes, seconds, milliseconds)
    }

}