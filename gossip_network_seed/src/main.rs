/// Main code for Seed Node

mod tokio_seed;
mod file_reader;

use tokio_seed::Seed;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::spawn;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Reads the IPs and port for each seeed from the config.txt file
    let file_path = "./src/config.txt";
    let (ips, ports) = file_reader::read_file(file_path)?;

    println!("IP Addresses: {:?}", ips);
    println!("Ports: {:?}", ports);
    
    let mut handles = vec![];
    let mut seed_no: i32 = 1;
    // Starts a listener for each SEed.
    for i in 0..ips.len() {
        let seed = Arc::new(Mutex::new(Seed::new(seed_no)));
        let seed_ip = ips[i].clone();
        let seed_port = ports[i].clone();
        // Creating a shared reference to the seed
        let seed_clone = Arc::clone(&seed);
        // spawning a thread for listening on each seed.
        let handle = spawn(async move {
            Seed::start_listener(seed_clone, seed_ip, seed_port).await;
        });
        
        seed_no+=1;
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    Ok(())
}