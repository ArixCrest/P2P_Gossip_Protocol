use std::fs::File;
use std::io::{self, BufRead, BufReader};

// logic for reading IPs and Ports from a file.
pub fn read_file(file_path: &str) -> Result<(Vec<String>, Vec<String>), io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut ips = Vec::new();
    let mut ports = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.trim().split_whitespace().collect();

        if parts.len() == 2 {
            let ip_address = parts[0].to_string();
            let port = parts[1].to_string();
            ips.push(ip_address);
            ports.push(port);
        } else {
            println!("Invalid line format: {}", line);
        }
    }

    Ok((ips, ports))
}