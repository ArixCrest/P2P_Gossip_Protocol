// Importing necessary packages
use std::collections::{HashSet, HashMap};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Duration, sleep};
use tokio::sync::Mutex;
use std::sync::Arc;

// importing necessary files
use crate::tokio_peer::Peer;
use crate::utils::parse_and_convert_to_ms;

//Constants for specific reply
const LIVENESS_REQUEST: &str = "LIVENESS_REQUEST";
const LIVENESS_REPLY: &str = "LIVENESS_REPLY";

// This function broadcasts the message to all connected nodes
pub async fn broadcast_message(connected_nodes: &HashSet<String>, message: String) {
    for connected_node in connected_nodes {
        let address = connected_node; 
        // Create a TCP connection and write the broadcast message to all the peers.
        match TcpStream::connect(address).await {
            Ok(mut stream) => {
                let _ = stream.write_all(message.as_bytes()).await;
                // println!("Message sent to {}: {}", connected_node, message);
            }
            Err(err) => {
                eprintln!("Error connecting to {}: {}", connected_node, err);
            }
        }
    }
}

// This function establishes a TCP connection and sends a liveness request.
pub async fn send_liveness_request(target_node: &String, message: String) {
    match TcpStream::connect(target_node).await {
        Ok(mut stream) => {
            let _ = stream.write_all(message.as_bytes()).await;
            // println!("Message sent to {}: {}", connected_node, message);
        }
        Err(err) => {
            eprintln!("Error connecting to {}: {}", target_node, err);
        }
    }
}
// This function establishes a TCP connection and sends a liveness reply.
pub async fn send_liveness_reply(target_node: &String, message: String) {
    match TcpStream::connect(target_node).await {
        Ok(mut stream) => {
            let _ = stream.write_all(message.as_bytes()).await;
            // println!("Message sent to {}: {}", connected_node, message);
        }
        Err(err) => {
            eprintln!("Error connecting to {}: {}", target_node, err);
        }
    }
}
// Checks for liveness between a shared peer reference and target node every 13 seconds.
pub async fn check_liveness(peer: Arc<Mutex<Peer>>, target: String){

    loop{
        let peer_guard = peer.lock().await;
        // check if the node is still considered alive or not.
        if peer_guard.connected_nodes.contains(&target){
            let message = format!("{}|{}|{}", LIVENESS_REQUEST, peer_guard.elapsed_time(), peer_guard.local_addr);
            // send liveness request
            send_liveness_request(&target, message).await;
            drop(peer_guard);
        }
        else{
            drop(peer_guard);
            break;
        }
        // sleep for 13 secs.
        sleep(Duration::from_secs(13)).await;
    }
}

// Creates a Listener, checks for liveness and responds to messages
pub async fn spawn_listener(peer: Arc<Mutex<Peer>>) {
    // Acquire a lock on peer guard to bind the ip and port.
    let peer_guard = peer.lock().await;
    let listener = TcpListener::bind(&peer_guard.local_addr).await.unwrap();
    println!("Peer {}: Listening on {}", peer_guard.peer_no, peer_guard.local_addr);
    // drop the lock.
    drop(peer_guard);

    // intialize a connection times which tracks when was the last liveness request received from a peer.
    let connection_times = Arc::new(Mutex::new(HashMap::<String, i32>::new()));
    for connected_node in &peer.lock().await.connected_nodes {
        connection_times.lock().await.insert(connected_node.to_string().clone(), 0);
    }

    // Checks for connection timeout by spawning a thread which loops every 14 seconds.
    {
        // create a shared reference from connection times and peer.
        let connection_times_clones = connection_times.clone();
        let peer_clone = peer.clone();
        tokio::spawn(async move {
            // ... existing code within the task
            sleep(Duration::from_secs(2)).await; // Pause for 1 second
    
            loop {
                // Check the condition every second
                let cur_time = parse_and_convert_to_ms(peer_clone.lock().await.elapsed_time().as_str());
                let mut nodes_to_remove: Vec<String> = Vec::new();
                // Aquire lock on connection_times shared reference
                let mut connection_times_guard = connection_times_clones.lock().await;
                
                for (key, value) in connection_times_guard.iter() {
                    // println!("Key: {}, Value: {}", key, value);
                    let prev_time: i32 = *value;
                    // check if the last liveness reply was more than 39 seconds ago.
                    if cur_time - prev_time >39000{

                        // get a lock on peer reference and remvoe the connection.
                        let mut peer_guard = peer_clone.lock().await;
                        peer_guard.connected_nodes.remove(key);
                        nodes_to_remove.push(key.clone());
                        peer_guard.declare_node_dead(key.clone()).await;
                    }
                }

                for key in nodes_to_remove {
                    // remove from connection times.
                    connection_times_guard.remove(&key);
                }
                // drop the guard on connection times.
                drop(connection_times_guard);
                sleep(Duration::from_secs(14)).await; // Pause for 14 second
            }
        });
    }
    // this is the main logic for listening and replying to requets.
    loop {
        // accept an incoming connection
        let (mut stream, _) = listener.accept().await.unwrap();
        // create a shared reference for connetion_times and peer.
        let peer_clone = peer.clone();
        let connection_times_clone = connection_times.clone();
        // spawn a thread for handling the incoming conneciton.
        tokio::spawn(async move {
            // reading and parsing the message from the buffer.
            let mut buffer = [0; 1024];
            let n = stream.read(&mut buffer).await.unwrap();
            let message = String::from_utf8_lossy(&buffer[..n]).to_string();
            let mut split_message: Vec<&str> = message.split('|').collect();
            split_message.iter_mut().for_each(|s| *s = s.trim());
            

            // Acquire the lockfor message handling
            let mut peer_guard = peer_clone.lock().await;
            let mut connection_times_guard = connection_times_clone.lock().await;
            // println!("Received message for peer {}: {}", peer_guard.local_addr, message);

            // Response logic for Liveness Reply
            if split_message[0] == LIVENESS_REQUEST{
                let response = format!("{}|{}|{}|{}",
                    LIVENESS_REPLY, split_message[1], split_message[2], peer_guard.local_addr);
                send_liveness_reply(&split_message[2].to_string(), response).await;
                // println!("Received Liveness Req\n");
            }
            // Response logic for Liveness Request
            else if split_message[0] == LIVENESS_REPLY {
                // update the connetion time for the node that replied.
                let cur_timestamp: i32 = parse_and_convert_to_ms(peer_guard.elapsed_time().as_str());
                let sender_ip = split_message[3].to_string();
                *connection_times_guard.get_mut(&sender_ip).unwrap() = cur_timestamp;
                // println!("Received Liveness Reply\n");
            }
            // Response logic for gossip message
            else if split_message.len() == 3 {
                let gossip_message = split_message[2];
                // Checks whether the message is duplicate or not.
                if peer_guard.message_list.contains(gossip_message) {
                    // println!("Duplicate Message\n");
                }else {
                    println!("Peer #{}: Received new message: {} from {} at timestamp: {}",
                        peer_guard.peer_no,gossip_message, split_message[1], split_message[0]);
                    peer_guard.message_list.insert(gossip_message.to_string());
                    let timestamp = peer_guard.elapsed_time();
                    let formatted_msg = format!("{}|{}|{}", timestamp, peer_guard.local_addr, gossip_message);
                    // broadcast the message to all the connected peers.
                    broadcast_message(&peer_guard.connected_nodes, formatted_msg.to_string()).await;
                }
            }
            // NO response when message is of incorrect format
            else{
                println!("Received Message of incorrect format\n");
            }
            // added due to too much fast leading to refused connection. can be fixed by raising the ulimit in terminal
            // sleep(Duration::from_secs(1)).await; 
        });
    }
}


// Code for idle listener to simulate a dead node
// Only Accepts messages and doesn't do anything.
pub async fn idle_listener(peer: Arc<Mutex<Peer>>){
    let listener = TcpListener::bind(&peer.lock().await.local_addr).await.unwrap();
    let peer_guard = peer.lock().await;
    println!("Peer {}: Idle Listening on {}", peer_guard.peer_no, peer_guard.local_addr);
    drop(peer_guard);
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            let n = stream.read(&mut buffer).await.unwrap();
            let _message = String::from_utf8_lossy(&buffer[..n]).to_string();

        });
    }
}