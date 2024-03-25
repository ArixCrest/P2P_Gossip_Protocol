# Gossip P2P Network

This repository contains code for implementing a gossip-based P2P Network. The system consists of two components: `gossip_network_peer` and `gossip_network_seed`, each representing a peer and a seed node implementation, respectively.

## Requirements
Ensure you have Rust installed. If not, you can download it from [rustup.rs](https://rustup.rs/) and follow the installation instructions.

## Running the Seed Code

To run the code, follow these steps:

1. Navigate to the directory `gossip_network_seed`.

2. Ensure that the `./src/config.txt` has the seed ip and port that you want. Having multiple Ips means the code will spawn multiple threads each listening on the port.

3. Inside the directory run the following command to build the files.
```bash
cargo build
```

4. Once the build is successful, run the main file using the below command.
```bash
cargo run | tee ./output.txt
```

5. After running the command you should see each seed Listening on their respective ports.

6. After running the program, you can check the `output.txt` file as well as the console for the requests received.

Note: Since there are multiple threads from the same program, to distinguish requests between seeds, each seed also prints at the start its identifier. Eg. `Seed #5: <response>...`

## Running the Peer Code
1. Navigate to the directory `gossip_network_peer`.

2. Ensure that the `./src/config.txt` inside the peer code directory has the seed ip and port same as that in of the seeds nodes you have selecting when creating seeds.

3. Currently the peer code spawns multiple peers(>1) each listening at ips and ports specifided in the file `peer_addr.txt` file, and of the multiple peers the last node is considered as dead to simulate a dead node request to the seed If you want to test this across multiple devices, then you can pass in single ip and port in the `./src/peer_addr.txt` file. and rest of the configuration will be the same.

4. Inside the directory run the following command to build the files.
```bash
cargo build
```

5. Once the build is successful, run the main file using the below command.
```bash
cargo run | tee ./output.txt
```

5. After running the command you should see each seed Listening on their respective ports.

6. After running the program, you can check the `output.txt` file as well as the console for the requests received.

Note: Since there are multiple threads from the same program, to distinguish requests between seeds, each seed also prints at the start its identifier. Eg. `Peer@<ip>: <response>...`, although for a single peer theres only one thread running for listener.

## Configuration

Both `gossip_network_peer` and `gossip_network_seed` components rely on configuration files located in the `src` directory. The `config.txt` file contains configuration parameters for seeds and `peer_addr.txt` contains the parameters from peers which are relevant to the operation of the program.

## Additional Notes

- Make sure to update the configuration files (`config.txt`, `peer_addr.txt`) with appropriate values before running the program.

- Ensure that necessary permissions and network access are granted to the program, especially if running on restricted environments.

- Refer to the code documentation and comments for more detailed information about the implementation and usage.
- If you are getting error about `too many open files` when running the command. It is recommended to increase the ulimit of your console by using the following command.(This ulimit worked for 10 peer instances and 11 seed instance.)
```bash
ulimit -n 4096
```

- The code contains the outputs when running the code in each of the `output.txt` file for reference.
