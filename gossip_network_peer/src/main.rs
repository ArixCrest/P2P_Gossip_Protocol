/// Main code for Peer Node

// Importing necessary files.
mod file_reader;
mod tokio_peer;
mod network;
mod utils;

// importing necessary modules
use tokio_peer::Peer;
use network::{check_liveness, spawn_listener, broadcast_message, idle_listener};
use utils::{get_ips, select_k_nodes};
use tokio::time::{Duration, sleep};
use tokio::sync::Mutex;
use std::sync::Arc;


#[tokio::main]
async fn main() {
    // Peer and Seed IP and Ports file path.
    let peer_ips_path = "./src/peer_addr.txt";
    let seed_ips_path = "./src/config.txt";

    // Reading the IP and Ports for Seed and Peers.
    let local_addresses = match get_ips(peer_ips_path) {
        Ok(local_addresses) => local_addresses,
        Err(err) => {
            eprintln!("Error obtaining peer IPs: {}", err);
            std::process::exit(1);
        }
    };
    let seed_nodes = match get_ips(seed_ips_path) {
        Ok(seed_nodes) => seed_nodes,
        Err(err) => {
            eprintln!("Error obtaining seed IPs: {}", err);
            std::process::exit(1);
        }
    };

    // Selecting K seeds for each Peer
    let tot_seeds = seed_nodes.len();
    let mut peers: Vec<Arc<Mutex<Peer>>> = Vec::new();
    let mut itr = 1;
    for local_address in local_addresses {

        let selected_seeds: Vec<String> = select_k_nodes(seed_nodes.clone(), tot_seeds/2+1);
        println!("Peer@{}: Selected seeds: {:?}", local_address, selected_seeds);
        let peer = Arc::new(Mutex::new(Peer::new(itr, local_address, selected_seeds)));

        // Join seed nodes using a lock guard.
        {
            let mut peer_guard = peer.lock().await;
            peer_guard.join_seed_nodes().await;
        }
        peers.push(peer);
        itr += 1;
    }

    // Select 4 distinct Peers for each Peer
    let tot_distinct_nodes = 4;
    for peer in &mut peers{
        let mut peer_guard = peer.lock().await;
        {
            peer_guard.query_connected_nodes().await;
            println!("Peer@{}: Peer nodes from Seeds: {:?}",
                peer_guard.local_addr, peer_guard.connected_nodes);
            // println!("Connected nodes of {}: {:?}", peer_guard.local_addr, peer_guard.connected_nodes);   
            if peer_guard.connected_nodes.len() > tot_distinct_nodes {
                let selected_nodes: Vec<_> = peer_guard.connected_nodes.iter().cloned().collect();
                let selected_nodes: Vec<_> = select_k_nodes(selected_nodes, tot_distinct_nodes);
                peer_guard.connected_nodes = selected_nodes.into_iter().collect();
            }
            println!("Peer@{}: Selected peer nodes: {:?}",
                peer_guard.local_addr, peer_guard.connected_nodes);   
        }
    }

    // Spawn Listener for each peer.
    let mut listeners = Vec::new();
    let peers_clone = peers.clone();
    for (index, peer) in peers_clone.iter().enumerate() {
        // For simulating a dead node, I took the last node as a dead node
        // The node is idle i.e doesn't response to any request
        if index == peers_clone.len() - 1 && peers_clone.len()>1 {
            let peer_clone = Arc::clone(peer);
            let listener_handle = tokio::spawn(idle_listener(peer_clone));
            listeners.push(listener_handle);
            continue;
        }
        // Spawn normal istener for other peers.
        let listener_handle = tokio::spawn(spawn_listener(peer.clone()));
        listeners.push(listener_handle);
    }

    sleep(Duration::from_secs(2)).await;

    //checks for liveness of peers connected to each respective peer.
    let peers_clone = peers.clone();
    for peer in &peers_clone {
        // Clone the peer and connected node for a shared reference.
        let peer_clone = Arc::clone(peer);
        let connected_nodes = peer_clone.lock().await.connected_nodes.clone();
        for connected_node in connected_nodes {
            let target_node = connected_node.clone();
            let peer_clone = Arc::clone(&peer);

            // Spawn a task for each pair of peer and connected node
            tokio::spawn(async move {
                check_liveness(peer_clone, target_node).await;
            });
        }
    }
    // Broadcasts the gossip messages to all peers every 5 seconds for 10 times.
    for _itr in 1..11 {
        let mut handles = vec![];

        for peer in peers.iter() {
            let peer_clone = Arc::clone(&peer);

            let handle = tokio::spawn(async move {

                // wait to obtain lock on the shared peer reference.
                let mut peer_guard = peer_clone.lock().await;
                let gossip = format!("Hello, this is peer @{}!", peer_guard.local_addr);
                // Add your own message to the message list.
                peer_guard.message_list.insert(gossip.to_string());
                let message = format!("{}|{}|{}", peer_guard.elapsed_time(), peer_guard.local_addr, gossip);
                // broadcast the message
                broadcast_message(&peer_guard.connected_nodes, message.clone()).await;
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete before moving to the next iteration
        for handle in handles {
            let _ = handle.await;
        }

        // Wait for 5 seconds before broadcasting the next message
        sleep(Duration::from_secs(5)).await;
    }
    

    tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");


}
