//! P2P server implementation for the blockchain network
//!
//! This module implements a peer-to-peer server for blockchain nodes to communicate.
//! It handles node discovery, block synchronization, transaction propagation,
//! and remote wallet operations through a standard binary protocol.

use crate::blockchain::block::Block;
use crate::blockchain::utxoset::UTXOSet;
use crate::crypto::fndsa::FnDsaCrypto;
use crate::crypto::ecdsa::EcdsaCrypto;
use crate::crypto::traits::CryptoProvider;
use crate::crypto::transaction::Transaction;
use crate::crypto::wallets::{Wallets, extract_encryption_type};
use crate::crypto::types::EncryptionType;
use crate::Result;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use bincode::{deserialize, serialize};
use failure::format_err;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

/// Size of command field in protocol messages
const CMD_LEN: usize = 12;
/// Protocol version
const VERSION: i32 = 1;
/// Default timeout for network operations (in seconds)
const NETWORK_TIMEOUT: u64 = 30;
/// Interval for node discovery (in seconds)
const DISCOVERY_INTERVAL: u64 = 300; // 5 minutes
/// Maximum number of peers to connect to
/// This constant is currently unused but reserved for future implementation.
const _MAX_PEERS: usize = 25;
/// Buffer size for reading from sockets
const READ_BUFFER_SIZE: usize = 8192;

/// Protocol message types
#[derive(Serialize, Deserialize, Debug, Clone)]
enum Message {
    /// Node addresses
    Addr(AddrMessage),
    /// Version information
    Version(VersionMessage),
    /// Transaction
    Tx(TxMessage),
    /// Request for data
    GetData(GetDataMessage),
    /// Request for blocks
    GetBlocks(GetBlocksMessage),
    /// Inventory announcement
    Inv(InvMessage),
    /// Block
    Block(BlockMessage),
    /// Transaction signing request
    SignRequest(SignRequestMessage),
    /// Transaction signing response
    SignResponse(SignResponseMessage),
    /// Ping message to check connectivity
    Ping(PingMessage),
    /// Pong response to ping
    Pong(PongMessage),
}

/// Block message containing a full block
#[derive(Serialize, Deserialize, Debug, Clone)]
struct BlockMessage {
    /// Sender node address
    addr_from: String,
    /// Block data
    block: Block,
}

/// Message to request blocks
#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetBlocksMessage {
    /// Sender node address
    addr_from: String,
    /// Optional hash to start from
    start_hash: Option<String>,
    /// Maximum number of blocks to return
    limit: Option<u32>,
}

/// Message to request specific data
#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetDataMessage {
    /// Sender node address
    addr_from: String,
    /// Type of data ("block" or "tx")
    kind: String,
    /// Hash of the requested item
    id: String,
}

/// Inventory message to announce available items
#[derive(Serialize, Deserialize, Debug, Clone)]
struct InvMessage {
    /// Sender node address
    addr_from: String,
    /// Type of items ("block" or "tx")
    kind: String,
    /// List of item hashes
    items: Vec<String>,
}

/// Transaction message
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TxMessage {
    /// Sender node address
    addr_from: String,
    /// Transaction data
    transaction: Transaction,
}

/// Version message for handshake
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct VersionMessage {
    /// Sender node address
    addr_from: String,
    /// Protocol version
    version: i32,
    /// Height of sender's blockchain
    best_height: i32,
    /// Timestamp of message
    timestamp: u64,
}

/// Address message for peer discovery
#[derive(Serialize, Deserialize, Debug, Clone)]
struct AddrMessage {
    /// Sender node address
    addr_from: String,
    /// List of known node addresses
    addresses: Vec<String>,
    /// Timestamp of message
    timestamp: u64,
}

/// Message to request transaction signing
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SignRequestMessage {
    /// Sender node address
    addr_from: String,
    /// Wallet address to sign with
    address: String,
    /// Transaction to sign
    transaction: Transaction,
}

/// Response to a signing request
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SignResponseMessage {
    /// Sender node address
    addr_from: String,
    /// Signed transaction (or original if failed)
    transaction: Transaction,
    /// Whether signing was successful
    success: bool,
    /// Error message if signing failed
    error_message: String,
}

/// Ping message to check node connectivity
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PingMessage {
    /// Sender node address
    addr_from: String,
    /// Nonce to match with pong
    nonce: u64,
    /// Timestamp of message
    timestamp: u64,
}

/// Pong response to a ping
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PongMessage {
    /// Sender node address
    addr_from: String,
    /// Nonce from the ping message
    nonce: u64,
    /// Timestamp of message
    timestamp: u64,
}

/// Node information
#[derive(Clone, Debug)]
struct PeerInfo {
    /// Network address
    _address: String,
    /// Last seen timestamp
    last_seen: Instant,
    /// Latest known blockchain height
    best_height: i32,
    /// Connection status
    status: PeerStatus,
}

/// Peer connection status
#[derive(Clone, Debug, PartialEq)]
enum PeerStatus {
    /// New peer, not yet contacted
    New,
    /// Successfully connected
    Connected,
    /// Failed to connect or disconnected
    Failed,
}

/// Internal server state
struct ServerInner {
    /// Known peers
    peers: HashMap<String, PeerInfo>,
    /// UTXO set and blockchain
    utxo: UTXOSet,
    /// Blocks being downloaded
    blocks_in_transit: Vec<String>,
    /// Unconfirmed transactions
    mempool: HashMap<String, Transaction>,
    /// Latest seen ping times (ms)
    ping_times: HashMap<String, u64>,
}

/// P2P blockchain server
pub struct Server {
    /// Node's network address
    node_address: String,
    /// Mining address (if this is a mining node)
    mining_address: String,
    /// Shared server state
    inner: Arc<Mutex<ServerInner>>,
    /// Server is running flag
    running: Arc<Mutex<bool>>,
}

impl Server {
    /// Creates a new server instance
    ///
    /// # Arguments
    ///
    /// * `host` - Host address to bind to
    /// * `port` - Port to listen on
    /// * `miner_address` - Mining reward address (if this is a mining node)
    /// * `bootstrap` - Optional bootstrap node to connect to
    /// * `utxo` - UTXO set
    ///
    /// # Returns
    ///
    /// A new server instance
    pub fn new(
        host: &str,
        port: &str,
        miner_address: &str,
        bootstrap: Option<&str>,
        utxo: UTXOSet,
    ) -> Result<Server> {
        let mut peers = HashMap::new();

        // Add bootstrap node if provided
        if let Some(bn) = bootstrap {
            peers.insert(
                bn.to_string(),
                PeerInfo {
                    _address: bn.to_string(),
                    last_seen: Instant::now(),
                    best_height: -1,
                    status: PeerStatus::New,
                },
            );
        }

        Ok(Server {
            node_address: format!("{}:{}", host, port),
            mining_address: miner_address.to_string(),
            inner: Arc::new(Mutex::new(ServerInner {
                peers,
                utxo,
                blocks_in_transit: Vec::new(),
                mempool: HashMap::new(),
                ping_times: HashMap::new(),
            })),
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// Starts the server and begins listening for connections
    ///
    /// This method starts the main server loop and several background tasks:
    /// - Node discovery thread
    /// - Mempool management thread
    /// - Block synchronization thread
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub fn start_server(&self) -> Result<()> {
        info!(
            "Starting server at {}, mining address: {}",
            &self.node_address, &self.mining_address
        );

        // Mark server as running
        {
            let mut running = self.running.lock().unwrap();
            *running = true;
        }

        // Clone references for background threads
        let server_discovery = self.clone();
        let server_mempool = self.clone();
        let server_sync = self.clone();

        // Start node discovery thread
        thread::spawn(move || {
            info!("Starting node discovery thread");
            while *server_discovery.running.lock().unwrap() {
                if let Err(e) = server_discovery.discover_nodes() {
                    error!("Node discovery error: {}", e);
                }
                thread::sleep(Duration::from_secs(DISCOVERY_INTERVAL));
            }
        });

        // Start mempool management thread
        thread::spawn(move || {
            info!("Starting mempool management thread");
            while *server_mempool.running.lock().unwrap() {
                if !server_mempool.mining_address.is_empty() {
                    if let Err(e) = server_mempool.process_mempool() {
                        error!("Mempool processing error: {}", e);
                    }
                }
                thread::sleep(Duration::from_secs(10));
            }
        });

        // Start block synchronization thread if we're a new node
        thread::spawn(move || {
            info!("Starting initial block synchronization");
            thread::sleep(Duration::from_secs(5)); // Give time for server to start

            if let Ok(height) = server_sync.get_best_height() {
                if height == -1 {
                    if let Err(e) = server_sync.synchronize_blockchain() {
                        error!("Initial blockchain sync error: {}", e);
                    }
                } else {
                    // We already have blocks, just announce our version
                    let peers = server_sync.get_peers();
                    for peer in peers {
                        if let Err(e) = server_sync.send_version(&peer) {
                            error!("Failed to send version to {}: {}", peer, e);
                        }
                    }
                }
            }
        });

        // Start main server loop
        let listener = match TcpListener::bind(&self.node_address) {
            Ok(listener) => listener,
            Err(e) => {
                error!("Failed to bind to {}: {}", self.node_address, e);
                return Err(format_err!(
                    "Failed to bind to {}: {}",
                    self.node_address,
                    e
                ));
            }
        };

        info!("Server listening for connections");

        for stream in listener.incoming() {
            if !*self.running.lock().unwrap() {
                break;
            }

            match stream {
                Ok(stream) => {
                    let server_conn = self.clone();
                    thread::spawn(move || {
                        if let Err(e) = server_conn.handle_connection(stream) {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Connection accept error: {}", e);
                }
            }
        }

        // Mark server as stopped
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        Ok(())
    }

    /// Stops the server gracefully
    pub fn stop_server(&self) -> Result<()> {
        info!("Stopping server");
        let mut running = self.running.lock().unwrap();
        *running = false;
        Ok(())
    }

    /// Sends a transaction to the network
    ///
    /// # Arguments
    ///
    /// * `tx` - Transaction to send
    /// * `utxoset` - UTXO set
    /// * `target_addr` - Target node address
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub fn send_transaction(tx: &Transaction, utxoset: UTXOSet, target_addr: &str) -> Result<()> {
        let server = Server::new("0.0.0.0", "0", "", None, utxoset)?;
        server.send_tx(target_addr, tx)?;
        Ok(())
    }

    /// Discovers and connects to new nodes
    fn discover_nodes(&self) -> Result<()> {
        let peers = self.get_peers();

        if peers.is_empty() {
            warn!("No peers available for discovery");
            return Ok(());
        }

        // Ask each connected peer for their known addresses
        for peer in peers {
            if let Err(e) = self.send_get_addr(&peer) {
                warn!("Failed to request addresses from {}: {}", peer, e);
                continue;
            }
        }

        Ok(())
    }

    /// Processes transactions in the mempool
    fn process_mempool(&self) -> Result<()> {
        let mempool = self.get_mempool();

        if mempool.is_empty() {
            return Ok(());
        }

        debug!("Processing mempool with {} transactions", mempool.len());

        // Collect valid transactions
        let mut txs = Vec::new();

        for tx in mempool.values() {
            if self.verify_tx(tx)? {
                txs.push(tx.clone());
            } else {
                warn!("Invalid transaction in mempool: {}", tx.id);
            }
        }

        if txs.is_empty() {
            return Ok(());
        }

        info!("Mining new block with {} transactions", txs.len());

        // Create coinbase transaction
        let cbtx = Transaction::new_coinbase(self.mining_address.clone(), String::from("reward!"))?;
        txs.push(cbtx);

        // Mine block
        let new_block = self.mine_block(txs)?;

        // Update UTXO set
        self.update_utxo(&new_block)?;

        // Announce block to peers
        let peers = self.get_peers();
        for peer in peers {
            if peer != self.node_address {
                self.send_inv(&peer, "block", vec![new_block.get_hash()])?;
            }
        }

        // Clear processed transactions from mempool
        self.clear_mempool();

        info!("New block mined: {}", new_block.get_hash());

        Ok(())
    }

    /// Synchronizes blockchain with peers
    fn synchronize_blockchain(&self) -> Result<()> {
        info!("Synchronizing blockchain with peers");

        let peers = self.get_peers();
        if peers.is_empty() {
            warn!("No peers available for synchronization");
            return Ok(());
        }

        // Request blocks from all peers
        for peer in peers {
            if let Err(e) = self.send_get_blocks(&peer) {
                warn!("Failed to request blocks from {}: {}", peer, e);
            }
        }

        Ok(())
    }

    /// Sends a request for peer addresses
    fn send_get_addr(&self, addr: &str) -> Result<()> {
        info!("Requesting peer addresses from {}", addr);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let msg = AddrMessage {
            addr_from: self.node_address.clone(),
            addresses: Vec::new(),
            timestamp: now,
        };

        self.send_message(addr, "addr", &msg)
    }

    /// Sends our known peer addresses
    fn send_addr(&self, addr: &str) -> Result<()> {
        info!("Sending peer addresses to {}", addr);

        let peers = self.get_peers();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let msg = AddrMessage {
            addr_from: self.node_address.clone(),
            addresses: peers,
            timestamp: now,
        };

        self.send_message(addr, "addr", &msg)
    }

    /// Sends version information to a peer
    fn send_version(&self, addr: &str) -> Result<()> {
        info!("Sending version info to {}", addr);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let msg = VersionMessage {
            addr_from: self.node_address.clone(),
            version: VERSION,
            best_height: self.get_best_height()?,
            timestamp: now,
        };

        self.send_message(addr, "version", &msg)
    }

    /// Sends block inventory to a peer
    fn send_inv(&self, addr: &str, kind: &str, items: Vec<String>) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }

        info!(
            "Sending inventory to {}: {} {} items",
            addr,
            items.len(),
            kind
        );

        let msg = InvMessage {
            addr_from: self.node_address.clone(),
            kind: kind.to_string(),
            items,
        };

        self.send_message(addr, "inv", &msg)
    }

    /// Sends a request for blocks
    fn send_get_blocks(&self, addr: &str) -> Result<()> {
        info!("Requesting blocks from {}", addr);

        let msg = GetBlocksMessage {
            addr_from: self.node_address.clone(),
            start_hash: None,
            limit: None,
        };

        self.send_message(addr, "getblocks", &msg)
    }

    /// Sends a request for specific data
    fn send_get_data(&self, addr: &str, kind: &str, id: &str) -> Result<()> {
        info!("Requesting {} data from {}: {}", kind, addr, id);

        let msg = GetDataMessage {
            addr_from: self.node_address.clone(),
            kind: kind.to_string(),
            id: id.to_string(),
        };

        self.send_message(addr, "getdata", &msg)
    }

    /// Sends a block to a peer
    fn send_block(&self, addr: &str, block: &Block) -> Result<()> {
        info!("Sending block to {}: {}", addr, block.get_hash());

        let msg = BlockMessage {
            addr_from: self.node_address.clone(),
            block: block.clone(),
        };

        self.send_message(addr, "block", &msg)
    }

    /// Sends a transaction to a peer
    pub fn send_tx(&self, addr: &str, tx: &Transaction) -> Result<()> {
        info!("Sending transaction to {}: {}", addr, tx.id);

        let msg = TxMessage {
            addr_from: self.node_address.clone(),
            transaction: tx.clone(),
        };

        self.send_message(addr, "tx", &msg)
    }

    /// Sends a ping to check connectivity
    fn _send_ping(&self, addr: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        // Generate random nonce
        let nonce = rand::random::<u64>();

        let msg = PingMessage {
            addr_from: self.node_address.clone(),
            nonce,
            timestamp: now,
        };

        self.send_message(addr, "ping", &msg)
    }

    /// Responds to a ping with a pong
    fn send_pong(&self, addr: &str, nonce: u64) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let msg = PongMessage {
            addr_from: self.node_address.clone(),
            nonce,
            timestamp: now,
        };

        self.send_message(addr, "pong", &msg)
    }

    /// Generic method to send a message to a peer
    fn send_message<T: Serialize>(&self, addr: &str, cmd: &str, payload: &T) -> Result<()> {
        if addr == self.node_address {
            return Ok(());
        }

        let cmd_bytes = cmd_to_bytes(cmd);
        let payload_bytes = serialize(payload)?;
        let mut message = Vec::with_capacity(CMD_LEN + payload_bytes.len());

        message.extend_from_slice(&cmd_bytes);
        message.extend_from_slice(&payload_bytes);

        self.send_data(addr, &message)
    }

    /// Sends raw data to a peer
    fn send_data(&self, addr: &str, data: &[u8]) -> Result<()> {
        if addr == self.node_address {
            return Ok(());
        }

        let mut stream = match TcpStream::connect_timeout(
            &addr.parse::<SocketAddr>()?,
            Duration::from_secs(NETWORK_TIMEOUT),
        ) {
            Ok(stream) => stream,
            Err(e) => {
                warn!("Failed to connect to {}: {}", addr, e);
                self.mark_peer_failed(addr);
                return Err(format_err!("Connection failed: {}", e));
            }
        };

        stream.set_write_timeout(Some(Duration::from_secs(NETWORK_TIMEOUT)))?;

        match stream.write_all(data) {
            Ok(_) => {
                stream.flush()?;
                self.update_peer_status(addr, PeerStatus::Connected);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to send data to {}: {}", addr, e);
                self.mark_peer_failed(addr);
                Err(format_err!("Send failed: {}", e))
            }
        }
    }

    /// Handles incoming connections
    fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        let peer_addr = match stream.peer_addr() {
            Ok(addr) => addr.to_string(),
            Err(_) => "unknown".to_string(),
        };

        info!("Handling connection from {}", peer_addr);

        stream.set_read_timeout(Some(Duration::from_secs(NETWORK_TIMEOUT)))?;

        let mut buffer = vec![0; READ_BUFFER_SIZE];

        let bytes_read = match stream.read(&mut buffer) {
            Ok(count) => {
                if count == 0 {
                    return Err(format_err!("Empty message from {}", peer_addr));
                }
                count
            }
            Err(e) => {
                return Err(format_err!("Read error from {}: {}", peer_addr, e));
            }
        };

        buffer.truncate(bytes_read);

        if buffer.len() < CMD_LEN {
            return Err(format_err!("Message too short from {}", peer_addr));
        }

        let cmd_bytes = &buffer[..CMD_LEN];
        let cmd = decode_command(cmd_bytes)?;
        let payload = &buffer[CMD_LEN..];

        debug!("Received command '{}' from {}", cmd, peer_addr);

        self.process_message(&cmd, payload, &peer_addr, &mut stream)
    }

    /// Processes a received message
    fn process_message(
        &self,
        cmd: &str,
        payload: &[u8],
        peer_addr: &str,
        stream: &mut TcpStream,
    ) -> Result<()> {
        match cmd {
            "version" => {
                let msg: VersionMessage = deserialize(payload)?;
                self.handle_version(msg, peer_addr)?;
            }
            "addr" => {
                let msg: AddrMessage = deserialize(payload)?;
                self.handle_addr(msg)?;
            }
            "block" => {
                let msg: BlockMessage = deserialize(payload)?;
                self.handle_block(msg)?;
            }
            "inv" => {
                let msg: InvMessage = deserialize(payload)?;
                self.handle_inv(msg)?;
            }
            "getblocks" => {
                let msg: GetBlocksMessage = deserialize(payload)?;
                self.handle_get_blocks(msg)?;
            }
            "getdata" => {
                let msg: GetDataMessage = deserialize(payload)?;
                self.handle_get_data(msg)?;
            }
            "tx" => {
                let msg: TxMessage = deserialize(payload)?;
                self.handle_tx(msg)?;
            }
            "ping" => {
                let msg: PingMessage = deserialize(payload)?;
                self.handle_ping(msg)?;
            }
            "pong" => {
                let msg: PongMessage = deserialize(payload)?;
                self.handle_pong(msg)?;
            }
            "signreq" => {
                let msg: SignRequestMessage = deserialize(payload)?;
                let response = self.handle_sign_request(msg)?;

                // Direct response needed
                let cmd_bytes = cmd_to_bytes("signres");
                let response_bytes = serialize(&response)?;
                let mut message = Vec::with_capacity(CMD_LEN + response_bytes.len());

                message.extend_from_slice(&cmd_bytes);
                message.extend_from_slice(&response_bytes);

                stream.write_all(&message)?;
                stream.flush()?;
            }
            _ => {
                warn!("Unknown command '{}' from {}", cmd, peer_addr);
            }
        }

        Ok(())
    }

    /// Handles version messages
    fn handle_version(&self, msg: VersionMessage, peer_addr: &str) -> Result<()> {
        info!(
            "Received version from {}: v{}, height {}",
            msg.addr_from, msg.version, msg.best_height
        );

        // Update peer info
        self.update_peer(&msg.addr_from, msg.best_height, PeerStatus::Connected);

        // Compare blockchain heights
        let my_height = self.get_best_height()?;

        if my_height < msg.best_height {
            // Our chain is shorter, request blocks
            info!(
                "Our blockchain ({}) is behind {} ({})",
                my_height, msg.addr_from, msg.best_height
            );
            self.send_get_blocks(&msg.addr_from)?;
        } else if my_height > msg.best_height {
            // Our chain is longer, send our version
            info!(
                "Our blockchain ({}) is ahead of {} ({})",
                my_height, msg.addr_from, msg.best_height
            );
            self.send_version(&msg.addr_from)?;
        }

        // Send address list if peer is new
        if msg.addr_from != peer_addr {
            // This means the peer is using a different address for its node_address
            // than the socket address we're communicating with
            self.update_peer(peer_addr, msg.best_height, PeerStatus::Connected);
        }

        // Share our known addresses
        self.send_addr(&msg.addr_from)?;

        Ok(())
    }

    /// Handles address messages
    fn handle_addr(&self, msg: AddrMessage) -> Result<()> {
        info!(
            "Received {} addresses from {}",
            msg.addresses.len(),
            msg.addr_from
        );

        for addr in msg.addresses {
            if addr != self.node_address && !self.is_peer_known(&addr) {
                info!("Discovered new peer: {}", addr);
                self.add_peer(&addr);

                // Send version to new peer
                if let Err(e) = self.send_version(&addr) {
                    warn!("Failed to send version to new peer {}: {}", addr, e);
                }
            }
        }

        Ok(())
    }

    /// Handles block messages
    fn handle_block(&self, msg: BlockMessage) -> Result<()> {
        info!(
            "Received block from {}: {}",
            msg.addr_from,
            msg.block.get_hash()
        );

        // Add block to our chain
        self.add_block(msg.block.clone())?;

        // Process any blocks in transit
        let mut in_transit = self.get_blocks_in_transit();
        if !in_transit.is_empty() {
            let block_hash = in_transit.remove(0);
            self.set_blocks_in_transit(in_transit);

            // Request next block
            self.send_get_data(&msg.addr_from, "block", &block_hash)?;
        } else {
            // No more blocks in transit, reindex UTXO
            info!("Blockchain sync complete, reindexing UTXO set");
            self.reindex_utxo()?;
        }

        Ok(())
    }

    /// Handles inventory messages
    fn handle_inv(&self, msg: InvMessage) -> Result<()> {
        info!(
            "Received inventory from {}: {} {} items",
            msg.addr_from,
            msg.items.len(),
            msg.kind
        );

        if msg.items.is_empty() {
            return Ok(());
        }

        match msg.kind.as_str() {
            "block" => {
                // Save block hashes for later processing
                let mut blocks_in_transit = self.get_blocks_in_transit();

                // Add the first block to request immediately
                if !msg.items.is_empty() {
                    let block_hash = &msg.items[0];
                    self.send_get_data(&msg.addr_from, "block", block_hash)?;

                    // Add remaining blocks to the in-transit queue
                    for hash in msg.items.iter().skip(1) {
                        if !blocks_in_transit.contains(hash) {
                            blocks_in_transit.push(hash.clone());
                        }
                    }

                    self.set_blocks_in_transit(blocks_in_transit);
                }
            }
            "tx" => {
                // Request unknown transactions
                for tx_id in &msg.items {
                    if !self.has_transaction(tx_id) {
                        self.send_get_data(&msg.addr_from, "tx", tx_id)?;
                    }
                }
            }
            _ => {
                warn!("Unknown inventory type: {}", msg.kind);
            }
        }

        Ok(())
    }

    /// Handles getblocks messages
    fn handle_get_blocks(&self, msg: GetBlocksMessage) -> Result<()> {
        info!("Received get blocks request from {}", msg.addr_from);

        // Get our block hashes
        let block_hashes = self.get_block_hashes();

        // Send inventory of our blocks
        self.send_inv(&msg.addr_from, "block", block_hashes)?;

        Ok(())
    }

    /// Handles getdata messages
    fn handle_get_data(&self, msg: GetDataMessage) -> Result<()> {
        info!(
            "Received get data request from {} for {} {}",
            msg.addr_from, msg.kind, msg.id
        );

        match msg.kind.as_str() {
            "block" => {
                // Send requested block
                if let Ok(block) = self.get_block(&msg.id) {
                    self.send_block(&msg.addr_from, &block)?;
                } else {
                    warn!("Block not found: {}", msg.id);
                }
            }
            "tx" => {
                // Send requested transaction
                if let Some(tx) = self.get_mempool_tx(&msg.id) {
                    self.send_tx(&msg.addr_from, &tx)?;
                } else {
                    warn!("Transaction not found in mempool: {}", msg.id);
                }
            }
            _ => {
                warn!("Unknown data type requested: {}", msg.kind);
            }
        }

        Ok(())
    }

    /// Handles transaction messages
    fn handle_tx(&self, msg: TxMessage) -> Result<()> {
        let tx_id = &msg.transaction.id;
        info!("Received transaction from {}: {}", msg.addr_from, tx_id);

        // Add to mempool
        self.add_to_mempool(msg.transaction.clone());

        // Relay to other peers
        let peers = self.get_peers();
        for peer in peers {
            if peer != self.node_address && peer != msg.addr_from {
                self.send_inv(&peer, "tx", vec![tx_id.clone()])?;
            }
        }

        // If we're a mining node, process mempool
        if !self.mining_address.is_empty() {
            self.process_mempool()?;
        }

        Ok(())
    }

    /// Handles ping messages
    fn handle_ping(&self, msg: PingMessage) -> Result<()> {
        debug!("Received ping from {}, nonce {}", msg.addr_from, msg.nonce);

        // Send pong response
        self.send_pong(&msg.addr_from, msg.nonce)?;

        Ok(())
    }

    /// Handles pong messages
    fn handle_pong(&self, msg: PongMessage) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let rtt = now.saturating_sub(msg.timestamp);

        debug!("Received pong from {}, RTT: {}ms", msg.addr_from, rtt);

        // Update ping time
        self.update_ping_time(&msg.addr_from, rtt);

        Ok(())
    }

    /// Handles transaction signing requests
    fn handle_sign_request(&self, msg: SignRequestMessage) -> Result<SignResponseMessage> {
        info!(
            "Received sign request from {} for wallet {}",
            msg.addr_from, msg.address
        );

        // Load wallets
        let wallets = match Wallets::new() {
            Ok(wallets) => wallets,
            Err(e) => {
                return Ok(SignResponseMessage {
                    addr_from: self.node_address.clone(),
                    transaction: msg.transaction.clone(),
                    success: false,
                    error_message: format!("Failed to load wallets: {}", e),
                });
            }
        };

        // Find requested wallet
        let wallet = match wallets.get_wallet(&msg.address) {
            Some(wallet) => wallet,
            None => {
                return Ok(SignResponseMessage {
                    addr_from: self.node_address.clone(),
                    transaction: msg.transaction.clone(),
                    success: false,
                    error_message: format!("Wallet not found: {}", msg.address),
                });
            }
        }
        let mut tx = msg.transaction.clone();
        
        // Extract encryption type from wallet address
        let (_, encryption_type) = match extract_encryption_type(&msg.address) {
            Ok(result) => result,
            Err(e) => {
                return Ok(SignResponseMessage {
                    addr_from: self.node_address.clone(),
                    transaction: msg.transaction.clone(),
                    success: false,
                    error_message: format!("Failed to extract encryption type: {}", e),
                });
            }
        };

        // Create appropriate crypto provider
        let result = match encryption_type {
            EncryptionType::FNDSA => {
                let crypto = FnDsaCrypto;
                self.sign_transaction(&mut tx, &wallet.secret_key, &crypto)
            }
            EncryptionType::ECDSA => {
                let crypto = EcdsaCrypto;
                self.sign_transaction(&mut tx, &wallet.secret_key, &crypto)
            }
        };

        match result {
            Ok(_) => {
                info!("Successfully signed transaction for {}", msg.address);
                Ok(SignResponseMessage {
                    addr_from: self.node_address.clone(),
                    transaction: tx,
                    success: true,
                    error_message: String::new(),
                })
            }
            Err(e) => {
                warn!("Failed to sign transaction: {}", e);
                Ok(SignResponseMessage {
                    addr_from: self.node_address.clone(),
                    transaction: msg.transaction,
                    success: false,
                    error_message: format!("Signing error: {}", e),
                })
            }
        }
    }

    /// Sends a transaction signing request to a remote node
    pub fn send_sign_request(
        &self,
        addr: &str,
        wallet_addr: &str,
        tx: &Transaction,
    ) -> Result<Transaction> {
        info!(
            "Sending sign request to {} for wallet {}",
            addr, wallet_addr
        );

        let msg = SignRequestMessage {
            addr_from: self.node_address.clone(),
            address: wallet_addr.to_string(),
            transaction: tx.clone(),
        };

        let cmd_bytes = cmd_to_bytes("signreq");
        let msg_bytes = serialize(&msg)?;
        let mut message = Vec::with_capacity(CMD_LEN + msg_bytes.len());

        message.extend_from_slice(&cmd_bytes);
        message.extend_from_slice(&msg_bytes);

        // Connect to remote node
        let mut stream = match TcpStream::connect_timeout(
            &addr.parse::<SocketAddr>()?,
            Duration::from_secs(NETWORK_TIMEOUT),
        ) {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to connect to {}: {}", addr, e);
                self.mark_peer_failed(addr);
                return Err(format_err!("Connection failed: {}", e));
            }
        };

        // Set timeouts
        stream.set_write_timeout(Some(Duration::from_secs(NETWORK_TIMEOUT)))?;
        stream.set_read_timeout(Some(Duration::from_secs(NETWORK_TIMEOUT)))?;

        // Send request
        stream.write_all(&message)?;
        stream.flush()?;

        // Read response
        let mut buffer = vec![0; READ_BUFFER_SIZE];
        let count = stream.read(&mut buffer)?;

        if count == 0 {
            return Err(format_err!("Empty response from {}", addr));
        }

        buffer.truncate(count);

        if buffer.len() < CMD_LEN {
            return Err(format_err!("Response too short from {}", addr));
        }

        let cmd_bytes = &buffer[..CMD_LEN];
        let cmd = decode_command(cmd_bytes)?;

        if cmd != "signres" {
            return Err(format_err!("Unexpected response command: {}", cmd));
        }

        let payload = &buffer[CMD_LEN..];
        let response: SignResponseMessage = deserialize(payload)?;

        if response.success {
            Ok(response.transaction)
        } else {
            Err(format_err!("Signing failed: {}", response.error_message))
        }
    }

    // Helper methods for peer management

    /// Adds a new peer
    fn add_peer(&self, addr: &str) {
        let mut inner = self.inner.lock().unwrap();

        if !inner.peers.contains_key(addr) && addr != &self.node_address {
            inner.peers.insert(
                addr.to_string(),
                PeerInfo {
                    _address: addr.to_string(),
                    last_seen: Instant::now(),
                    best_height: -1,
                    status: PeerStatus::New,
                },
            );
        }
    }

    /// Updates peer information
    fn update_peer(&self, addr: &str, height: i32, status: PeerStatus) {
        let mut inner = self.inner.lock().unwrap();

        if let Some(peer) = inner.peers.get_mut(addr) {
            peer.last_seen = Instant::now();
            peer.best_height = height;
            peer.status = status;
        } else if addr != self.node_address {
            inner.peers.insert(
                addr.to_string(),
                PeerInfo {
                    _address: addr.to_string(),
                    last_seen: Instant::now(),
                    best_height: height,
                    status,
                },
            );
        }
    }

    /// Updates peer status
    fn update_peer_status(&self, addr: &str, status: PeerStatus) {
        let mut inner = self.inner.lock().unwrap();

        if let Some(peer) = inner.peers.get_mut(addr) {
            peer.last_seen = Instant::now();
            peer.status = status;
        }
    }

    /// Marks a peer as failed
    fn mark_peer_failed(&self, addr: &str) {
        let mut inner = self.inner.lock().unwrap();

        if let Some(peer) = inner.peers.get_mut(addr) {
            peer.status = PeerStatus::Failed;
        }
    }

    /// Checks if a peer is known
    fn is_peer_known(&self, addr: &str) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.peers.contains_key(addr)
    }

    /// Gets all known peers
    fn get_peers(&self) -> Vec<String> {
        let inner = self.inner.lock().unwrap();

        inner
            .peers
            .iter()
            .filter(|(addr, info)| {
                // Only include connected peers and not ourselves
                info.status == PeerStatus::Connected && **addr != self.node_address
            })
            .map(|(addr, _)| addr.clone())
            .collect()
    }

    /// Updates ping time for a peer
    fn update_ping_time(&self, addr: &str, time: u64) {
        let mut inner = self.inner.lock().unwrap();
        inner.ping_times.insert(addr.to_string(), time);
    }

    // Helper methods for blockchain operations

    /// Gets the height of our blockchain
    fn get_best_height(&self) -> Result<i32> {
        let inner = self.inner.lock().unwrap();
        inner.utxo.blockchain.get_best_height()
    }

    /// Gets all block hashes in our blockchain
    fn get_block_hashes(&self) -> Vec<String> {
        let inner = self.inner.lock().unwrap();
        inner.utxo.blockchain.get_block_hashs()
    }

    /// Gets a block by hash
    fn get_block(&self, block_hash: &str) -> Result<Block> {
        let inner = self.inner.lock().unwrap();
        inner.utxo.blockchain.get_block(block_hash)
    }

    /// Adds a block to our blockchain
    fn add_block(&self, block: Block) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.utxo.blockchain.add_block(block)
    }

    /// Mines a new block
    fn mine_block(&self, txs: Vec<Transaction>) -> Result<Block> {
        let mut inner = self.inner.lock().unwrap();
        inner.utxo.blockchain.mine_block(txs)
    }

    /// Reindexes the UTXO set
    fn reindex_utxo(&self) -> Result<()> {
        let inner = self.inner.lock().unwrap();
        inner.utxo.reindex()
    }

    /// Updates the UTXO set with a new block
    fn update_utxo(&self, block: &Block) -> Result<()> {
        let inner = self.inner.lock().unwrap();
        inner.utxo.update(block)
    }

    /// Verifies a transaction
    fn verify_tx(&self, tx: &Transaction) -> Result<bool> {
        let inner = self.inner.lock().unwrap();
        inner.utxo.blockchain.verify_transacton(tx)
    }

    /// Signs a transaction
    fn sign_transaction(
        &self,
        tx: &mut Transaction,
        private_key: &[u8],
        crypto: &dyn CryptoProvider,
    ) -> Result<()> {
        let inner = self.inner.lock().unwrap();
        inner
            .utxo
            .blockchain
            .sign_transacton(tx, private_key, crypto)
    }

    // Helper methods for mempool management

    /// Adds a transaction to the mempool
    fn add_to_mempool(&self, tx: Transaction) {
        let mut inner = self.inner.lock().unwrap();
        inner.mempool.insert(tx.id.clone(), tx);
    }

    /// Gets the entire mempool
    fn get_mempool(&self) -> HashMap<String, Transaction> {
        let inner = self.inner.lock().unwrap();
        inner.mempool.clone()
    }

    /// Gets a transaction from the mempool
    fn get_mempool_tx(&self, tx_id: &str) -> Option<Transaction> {
        let inner = self.inner.lock().unwrap();
        inner.mempool.get(tx_id).cloned()
    }

    /// Checks if a transaction is in the mempool
    fn has_transaction(&self, tx_id: &str) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.mempool.contains_key(tx_id)
    }

    /// Clears the mempool
    fn clear_mempool(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.mempool.clear();
    }

    // Helper methods for block transit management

    /// Gets blocks in transit
    fn get_blocks_in_transit(&self) -> Vec<String> {
        let inner = self.inner.lock().unwrap();
        inner.blocks_in_transit.clone()
    }

    /// Sets blocks in transit
    fn set_blocks_in_transit(&self, blocks: Vec<String>) {
        let mut inner = self.inner.lock().unwrap();
        inner.blocks_in_transit = blocks;
    }
}

impl Clone for Server {
    fn clone(&self) -> Self {
        Server {
            node_address: self.node_address.clone(),
            mining_address: self.mining_address.clone(),
            inner: Arc::clone(&self.inner),
            running: Arc::clone(&self.running),
        }
    }
}

/// Converts a command string to a fixed-size byte array
fn cmd_to_bytes(cmd: &str) -> [u8; CMD_LEN] {
    let mut bytes = [0; CMD_LEN];
    for (i, b) in cmd.bytes().enumerate() {
        if i < CMD_LEN {
            bytes[i] = b;
        } else {
            break;
        }
    }
    bytes
}

/// Decodes a command from a byte array
fn decode_command(bytes: &[u8]) -> Result<String> {
    let mut cmd = String::new();
    for &b in bytes.iter().take(CMD_LEN) {
        if b != 0 {
            cmd.push(b as char);
        }
    }
    Ok(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::blockchain::Blockchain;
    use crate::crypto::types::EncryptionType;
    use crate::crypto::wallets::Wallets;

    #[test]
    fn test_cmd_conversion() {
        let cmd = "version";
        let bytes = cmd_to_bytes(cmd);
        let decoded = decode_command(&bytes).unwrap();
        assert_eq!(cmd, decoded);

        let cmd = "a_very_long_command_that_exceeds_length";
        let bytes = cmd_to_bytes(cmd);
        let decoded = decode_command(&bytes).unwrap();
        assert_eq!(&cmd[..CMD_LEN], decoded);
    }    #[test]
    fn test_server_creation() {
        use crate::config::DataContext;
        use std::path::PathBuf;
        
        // Use a test-specific data directory to avoid conflicts with existing data
        let test_context = DataContext::new(PathBuf::from("test_data_server"));
        
        // Create wallets with test context
        let mut wallets = Wallets::new_with_context(test_context.clone()).unwrap();
        let address = wallets.create_wallet(EncryptionType::FNDSA);
        wallets.save_all().unwrap();

        // Create blockchain with test context
        let bc = match Blockchain::new_with_context(test_context.clone()) {
            Ok(bc) => bc,
            Err(_) => Blockchain::create_blockchain_with_context(address, test_context.clone()).unwrap(),
        };

        let utxo_set = UTXOSet { blockchain: bc };
        let server = Server::new("127.0.0.1", "7000", "", None, utxo_set).unwrap();

        assert_eq!(server.node_address, "127.0.0.1:7000");
        assert_eq!(server.mining_address, "");

        let inner = server.inner.lock().unwrap();
        assert!(inner.peers.is_empty());
        
        // Clean up test data
        let _ = std::fs::remove_dir_all("test_data_server");
    }
}
