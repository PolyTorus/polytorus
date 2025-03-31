// src/network/server.rs
//! ブロックチェーンのネットワークサーバー実装

use crate::blockchain::block::Block;
use crate::blockchain::utxoset::UTXOSet;
use crate::crypto::fndsa::FnDsaCrypto;
use crate::crypto::transaction::Transaction;
use crate::crypto::wallets::Wallets;
use crate::crypto::types::{CryptoType, PrivateKey, PublicKey, Signature};
use crate::types::{BlockHash, TransactionId, Address};
use crate::errors::{BlockchainError, Result};
// use crate::network::message::*;
use crate::config::NetworkConfig;
use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::{HashMap, HashSet};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::*;
use std::thread;
use std::time::Duration;

/// メッセージタイプ
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    Addr,
    Block,
    Inv,
    GetBlock,
    GetData,
    Tx,
    Version,
    SignRequest,
    SignResponse,
}

/// ネットワークメッセージ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage<T> {
    pub message_type: MessageType,
    pub addr_from: String,
    pub payload: T,
}

/// バージョンメッセージのペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMessagePayload {
    pub version: u32,
    pub best_height: i32,
}

/// アドレスメッセージのペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddrMessagePayload {
    pub addresses: Vec<String>,
}

/// ブロックメッセージのペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMessagePayload {
    pub block: Block,
}

/// インベントリメッセージのペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvMessagePayload {
    pub kind: String,
    pub items: Vec<String>,
}

/// ブロック要求メッセージのペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBlocksMessagePayload {}

/// データ要求メッセージのペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDataMessagePayload {
    pub kind: String,
    pub id: String,
}

/// トランザクションメッセージのペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxMessagePayload {
    pub transaction: Transaction,
}

/// 署名要求メッセージのペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignRequestPayload {
    pub address: String,
    pub transaction: Transaction,
}

/// 署名応答メッセージのペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignResponsePayload {
    pub transaction: Transaction,
    pub success: bool,
    pub error_message: String,
}

/// ブロックチェーンのサーバー
pub struct Server {
    /// このノードのアドレス
    node_address: String,
    /// マイニングアドレス（報酬を受け取るアドレス）
    mining_address: Address,
    /// サーバーの内部状態
    inner: Arc<Mutex<ServerInner>>,
    /// ネットワーク設定
    config: NetworkConfig,
}

/// サーバーの内部状態
struct ServerInner {
    /// 既知のノードのセット
    known_nodes: HashSet<String>,
    /// UTXOセット
    utxo: UTXOSet,
    /// 転送中のブロック
    blocks_in_transit: Vec<BlockHash>,
    /// メモリプール（未承認トランザクション）
    mempool: HashMap<TransactionId, Transaction>,
}

impl Server {
    /// 新しいサーバーを作成
    pub fn new(
        host: &str,
        port: &str,
        miner_address: &Address,
        bootstrap: Option<&str>,
        utxo: UTXOSet,
        config: NetworkConfig,
    ) -> Result<Server> {
        let mut node_set = HashSet::new();
        
        if let Some(bn) = bootstrap {
            node_set.insert(bn.to_string());
        }
        
        Ok(Server {
            node_address: format!("{}:{}", host, port),
            mining_address: miner_address.clone(),
            inner: Arc::new(Mutex::new(ServerInner {
                known_nodes: node_set,
                utxo,
                blocks_in_transit: Vec::new(),
                mempool: HashMap::new(),
            })),
            config,
        })
    }

    /// サーバーを起動
    pub fn start_server(&self) -> Result<()> {
        let server1 = Server {
            node_address: self.node_address.clone(),
            mining_address: self.mining_address.clone(),
            inner: Arc::clone(&self.inner),
            config: self.config.clone(),
        };
        
        info!(
            "サーバーを起動: {}, マイニングアドレス: {}",
            &self.node_address, &self.mining_address
        );

        // 初期同期のためのスレッド
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            if server1.get_best_height()? == -1 {
                server1.request_blocks()
            } else {
                let nodes = server1.get_known_nodes();
                if !nodes.is_empty() {
                    let first = nodes.iter().next().unwrap();
                    server1.send_version(first)?;
                }
                Ok(())
            }
        });

        // TCP接続のリッスン
        let listener = TcpListener::bind(&self.node_address)
            .map_err(|e| BlockchainError::NetworkError(format!("TCPリスナーのバインドに失敗: {}", e)))?;
            
        info!("サーバーがリッスン中...");

        for stream in listener.incoming() {
            let stream = stream
                .map_err(|e| BlockchainError::NetworkError(format!("ストリームの受け入れに失敗: {}", e)))?;
                
            let server1 = Server {
                node_address: self.node_address.clone(),
                mining_address: self.mining_address.clone(),
                inner: Arc::clone(&self.inner),
                config: self.config.clone(),
            };
            
            thread::spawn(move || server1.handle_connection(stream));
        }

        Ok(())
    }

    /// トランザクションを送信
    pub fn send_transaction(tx: &Transaction, utxoset: UTXOSet, target_addr: &str, config: NetworkConfig) -> Result<()> {
        let server = Server::new(
            "0.0.0.0", 
            "7000", 
            &Address::empty(), 
            None, 
            utxoset,
            config,
        )?;
        
        server.send_tx(target_addr, tx)?;
        Ok(())
    }

    /* ------------------- 内部ヘルパー関数 ----------------------------------*/

    /// ノードを削除
    fn remove_node(&self, addr: &str) {
        self.inner.lock().unwrap().known_nodes.remove(addr);
    }

    /// ノードを追加
    fn add_nodes(&self, addr: &str) {
        self.inner
            .lock()
            .unwrap()
            .known_nodes
            .insert(String::from(addr));
    }

    /// 既知のノードを取得
    fn get_known_nodes(&self) -> HashSet<String> {
        self.inner.lock().unwrap().known_nodes.clone()
    }

    /// ノードが既知かどうかを確認
    fn node_is_known(&self, addr: &str) -> bool {
        self.inner.lock().unwrap().known_nodes.get(addr).is_some()
    }

    /// 転送中のブロックを置き換え
    fn replace_in_transit(&self, hashs: Vec<BlockHash>) {
        let bit = &mut self.inner.lock().unwrap().blocks_in_transit;
        bit.clone_from(&hashs);
    }

    /// 転送中のブロックを取得
    fn get_in_transit(&self) -> Vec<BlockHash> {
        self.inner.lock().unwrap().blocks_in_transit.clone()
    }

    /// メモリプールからトランザクションを取得
    fn get_mempool_tx(&self, id: &TransactionId) -> Option<Transaction> {
        self.inner.lock().unwrap().mempool.get(id).cloned()
    }

    /// メモリプール全体を取得
    fn get_mempool(&self) -> HashMap<TransactionId, Transaction> {
        self.inner.lock().unwrap().mempool.clone()
    }

    /// メモリプールにトランザクションを挿入
    fn insert_mempool(&self, tx: Transaction) {
        self.inner.lock().unwrap().mempool.insert(tx.id.clone(), tx);
    }

    /// メモリプールをクリア
    fn clear_mempool(&self) {
        self.inner.lock().unwrap().mempool.clear()
    }

    /// 最高ブロック高を取得
    fn get_best_height(&self) -> Result<i32> {
        self.inner.lock().unwrap().utxo.blockchain.get_best_height()
    }

    /// ブロックハッシュのリストを取得
    fn get_block_hashs(&self) -> Vec<BlockHash> {
        self.inner.lock().unwrap().utxo.blockchain.get_block_hashs()
            .into_iter()
            .map(|hash| BlockHash(hash))
            .collect()
    }

    /// ブロックを取得
    fn get_block(&self, block_hash: &BlockHash) -> Result<Block> {
        self.inner
            .lock()
            .unwrap()
            .utxo
            .blockchain
            .get_block(block_hash.as_str())
    }

    /// トランザクションを検証
    fn verify_tx(&self, tx: &Transaction) -> Result<bool> {
        self.inner
            .lock()
            .unwrap()
            .utxo
            .blockchain
            .verify_transacton(tx)
    }

    /// ブロックを追加
    fn add_block(&self, block: Block) -> Result<()> {
        self.inner.lock().unwrap().utxo.blockchain.add_block(block)
    }

    /// ブロックを採掘
    fn mine_block(&self, txs: Vec<Transaction>) -> Result<Block> {
        self.inner.lock().unwrap().utxo.blockchain.mine_block(txs)
    }

    /// UTXOセットを再インデックス
    fn utxo_reindex(&self) -> Result<()> {
        self.inner.lock().unwrap().utxo.reindex()
    }

    /* -----------------------------------------------------*/

    /// データを送信
    fn send_data(&self, addr: &str, data: &[u8]) -> Result<()> {
        if addr == &self.node_address {
            return Ok(());
        }
        
        let mut stream = match TcpStream::connect(addr) {
            Ok(s) => s,
            Err(e) => {
                warn!("接続失敗 {}: {}", addr, e);
                self.remove_node(addr);
                return Ok(());
            }
        };

        stream.write_all(data)
            .map_err(|e| BlockchainError::NetworkError(format!("データ送信エラー: {}", e)))?;

        info!("データを正常に送信しました");
        Ok(())
    }

    /// ブロックを要求
    fn request_blocks(&self) -> Result<()> {
        for node in self.get_known_nodes() {
            self.send_get_blocks(&node)?
        }
        Ok(())
    }

    /// ブロックを送信
    fn send_block(&self, addr: &str, b: &Block) -> Result<()> {
        info!("ブロックデータを送信: {} ブロックハッシュ: {}", addr, b.get_hash());
        
        let payload = BlockMessagePayload {
            block: b.clone(),
        };
        
        let message = NetworkMessage {
            message_type: MessageType::Block,
            addr_from: self.node_address.clone(),
            payload,
        };
        
        let data = serialize(&message)?;
        self.send_data(addr, &data)
    }

    /// アドレス情報を送信
    fn send_addr(&self, addr: &str) -> Result<()> {
        info!("アドレス情報を送信: {}", addr);
        
        let nodes = self.get_known_nodes().into_iter().collect();
        let payload = AddrMessagePayload {
            addresses: nodes,
        };
        
        let message = NetworkMessage {
            message_type: MessageType::Addr,
            addr_from: self.node_address.clone(),
            payload,
        };
        
        let data = serialize(&message)?;
        self.send_data(addr, &data)
    }

    /// インベントリを送信
    fn send_inv(&self, addr: &str, kind: &str, items: Vec<String>) -> Result<()> {
        info!(
            "インベントリメッセージを送信: {} 種類: {} データ: {:?}",
            addr, kind, items
        );
        
        let payload = InvMessagePayload {
            kind: kind.to_string(),
            items,
        };
        
        let message = NetworkMessage {
            message_type: MessageType::Inv,
            addr_from: self.node_address.clone(),
            payload,
        };
        
        let data = serialize(&message)?;
        self.send_data(addr, &data)
    }

    /// ブロック要求を送信
    fn send_get_blocks(&self, addr: &str) -> Result<()> {
        info!("ブロック要求メッセージを送信: {}", addr);
        
        let payload = GetBlocksMessagePayload {};
        
        let message = NetworkMessage {
            message_type: MessageType::GetBlock,
            addr_from: self.node_address.clone(),
            payload,
        };
        
        let data = serialize(&message)?;
        self.send_data(addr, &data)
    }

    /// データ要求を送信
    fn send_get_data(&self, addr: &str, kind: &str, id: &str) -> Result<()> {
        info!(
            "データ要求メッセージを送信: {} 種類: {} ID: {}",
            addr, kind, id
        );
        
        let payload = GetDataMessagePayload {
            kind: kind.to_string(),
            id: id.to_string(),
        };
        
        let message = NetworkMessage {
            message_type: MessageType::GetData,
            addr_from: self.node_address.clone(),
            payload,
        };
        
        let data = serialize(&message)?;
        self.send_data(addr, &data)
    }

    /// トランザクションを送信
    pub fn send_tx(&self, addr: &str, tx: &Transaction) -> Result<()> {
        info!("トランザクションを送信: {} txid: {}", addr, &tx.id);
        
        let payload = TxMessagePayload {
            transaction: tx.clone(),
        };
        
        let message = NetworkMessage {
            message_type: MessageType::Tx,
            addr_from: self.node_address.clone(),
            payload,
        };
        
        let data = serialize(&message)?;
        self.send_data(addr, &data)
    }

    /// バージョン情報を送信
    fn send_version(&self, addr: &str) -> Result<()> {
        info!("バージョン情報を送信: {}", addr);
        
        let payload = VersionMessagePayload {
            version: self.config.version as u32,
            best_height: self.get_best_height()?,
        };
        
        let message = NetworkMessage {
            message_type: MessageType::Version,
            addr_from: self.node_address.clone(),
            payload,
        };
        
        let data = serialize(&message)?;
        self.send_data(addr, &data)
    }

    /// バージョンメッセージを処理
    fn handle_version(&self, addr_from: String, payload: VersionMessagePayload) -> Result<()> {
        info!("バージョンメッセージを受信: {:?}", payload);
        
        let my_best_height = self.get_best_height()?;
        
        if my_best_height < payload.best_height {
            self.send_get_blocks(&addr_from)?;
        } else if my_best_height > payload.best_height {
            self.send_version(&addr_from)?;
        }

        self.send_addr(&addr_from)?;

        if !self.node_is_known(&addr_from) {
            self.add_nodes(&addr_from);
        }
        
        Ok(())
    }

    /// アドレスメッセージを処理
    fn handle_addr(&self, addresses: Vec<String>) -> Result<()> {
        info!("アドレスメッセージを受信: {:?}", addresses);
        
        for node in addresses {
            self.add_nodes(&node);
        }
        
        Ok(())
    }

    /// ブロックメッセージを処理
    fn handle_block(&self, addr_from: String, block: Block) -> Result<()> {
        info!(
            "ブロックメッセージを受信: {}, {}",
            addr_from,
            block.get_hash()
        );
        
        self.add_block(block)?;

        let mut in_transit = self.get_in_transit();
        
        if !in_transit.is_empty() {
            let block_hash = &in_transit[0];
            self.send_get_data(&addr_from, "block", block_hash.as_str())?;
            in_transit.remove(0);
            self.replace_in_transit(in_transit);
        } else {
            self.utxo_reindex()?;
        }

        Ok(())
    }

    /// インベントリメッセージを処理
    fn handle_inv(&self, addr_from: String, kind: String, items: Vec<String>) -> Result<()> {
        info!("インベントリメッセージを受信: {} {:?}", kind, items);
        
        if kind == "block" {
            let block_hash = &items[0];
            self.send_get_data(&addr_from, "block", block_hash)?;

            let mut new_in_transit = Vec::new();
            for b in &items {
                if b != block_hash {
                    new_in_transit.push(BlockHash(b.clone()));
                }
            }
            self.replace_in_transit(new_in_transit);
        } else if kind == "tx" {
            let txid = &items[0];
            let tx_id = TransactionId(txid.clone());
            
            match self.get_mempool_tx(&tx_id) {
                Some(tx) => {
                    if tx.id.is_empty() {
                        self.send_get_data(&addr_from, "tx", txid)?
                    }
                }
                None => self.send_get_data(&addr_from, "tx", txid)?,
            }
        }
        
        Ok(())
    }

    /// ブロック要求メッセージを処理
    fn handle_get_blocks(&self, addr_from: String) -> Result<()> {
        info!("ブロック要求メッセージを受信");
        
        let block_hashs = self.get_block_hashs();
        let block_hash_strs: Vec<String> = block_hashs.iter().map(|h| h.as_str().to_string()).collect();
        
        self.send_inv(&addr_from, "block", block_hash_strs)?;
        Ok(())
    }

    /// データ要求メッセージを処理
    fn handle_get_data(&self, addr_from: String, kind: String, id: String) -> Result<()> {
        info!("データ要求メッセージを受信: {} {}", kind, id);
        
        if kind == "block" {
            let block = self.get_block(&BlockHash(id))?;
            self.send_block(&addr_from, &block)?;
        } else if kind == "tx" {
            let tx = self.get_mempool_tx(&TransactionId(id)).ok_or(
                BlockchainError::InvalidTransaction("メモリプールにトランザクションが見つかりません".to_string())
            )?;
            self.send_tx(&addr_from, &tx)?;
        }
        
        Ok(())
    }

    /// トランザクションメッセージを処理
    fn handle_tx(&self, addr_from: String, transaction: Transaction) -> Result<()> {
        info!("トランザクションメッセージを受信: {} {}", addr_from, &transaction.id);
        
        self.insert_mempool(transaction.clone());

        let known_nodes = self.get_known_nodes();

        for node in known_nodes {
            if node != self.node_address && node != addr_from {
                self.send_inv(&node, "tx", vec![transaction.id.as_str().to_string()])?;
            }
        }

        // マイニングアドレスが設定されている場合はマイニングを開始
        if !self.mining_address.is_empty() {
            let mut mempool = self.get_mempool();
            debug!("現在のメモリプール: {:#?}", &mempool);

            if !mempool.is_empty() {
                loop {
                    let mut txs = Vec::new();

                    for tx in mempool.values() {
                        if self.verify_tx(tx)? {
                            txs.push(tx.clone());
                        }
                    }

                    if txs.is_empty() {
                        return Ok(());
                    }

                    // コインベーストランザクションを追加（マイニング報酬）
                    let cbtx = Transaction::new_coinbase(
                        self.mining_address.clone(), 
                        String::from("reward!"),
                        &self.inner.lock().unwrap().utxo.blockchain.config,
                    )?;
                    txs.push(cbtx);

                    // 処理済みトランザクションをメモリプールから削除
                    for tx in &txs {
                        mempool.remove(&tx.id);
                    }

                    // 新しいブロックを採掘
                    let new_block = self.mine_block(txs)?;
                    self.utxo_reindex()?;

                    // 新ブロックを他のノードに通知
                    for node in self.get_known_nodes() {
                        if node != self.node_address {
                            self.send_inv(&node, "block", vec![new_block.get_hash().as_str().to_string()])?;
                        }
                    }

                    if mempool.is_empty() {
                        break;
                    }
                }
                self.clear_mempool();
            }
        }

        Ok(())
    }

    /// リモート署名要求
    pub fn send_sign_request(
        &self,
        addr: &str,
        wallet_addr: &str,
        tx: &Transaction,
    ) -> Result<Transaction> {
        info!("署名要求を送信: {} ウォレット: {}", addr, wallet_addr);
        
        let payload = SignRequestPayload {
            address: wallet_addr.to_string(),
            transaction: tx.clone(),
        };
        
        let message = NetworkMessage {
            message_type: MessageType::SignRequest,
            addr_from: self.node_address.clone(),
            payload,
        };
        
        let data = serialize(&message)?;

        let mut stream = match TcpStream::connect(addr) {
            Ok(s) => s,
            Err(e) => {
                error!("接続失敗: {}", e);
                self.remove_node(addr);
                return Err(BlockchainError::NetworkError(format!("接続失敗: {}", e)));
            }
        };

        stream.set_read_timeout(Some(Duration::from_secs(30)))?;

        info!("リクエストデータを書き込み: {} バイト", data.len());

        stream.write_all(&data)?;
        stream.flush()?;

        let mut buffer = vec![0; 10240];
        info!("応答を待機中...");
        let count = stream.read(&mut buffer)?;
        buffer.truncate(count);

        info!("応答を受信: {} バイト", buffer.len());

        if count == 0 {
            return Err(BlockchainError::NetworkError("サーバーからの空の応答".to_string()));
        }

        let message: NetworkMessage<SignResponsePayload> = deserialize(&buffer)?;
        
        if message.message_type != MessageType::SignResponse {
            return Err(BlockchainError::NetworkError("予期しないレスポンス".to_string()));
        }
        
        if message.payload.success {
            Ok(message.payload.transaction)
        } else {
            Err(BlockchainError::InvalidSignature(format!(
                "トランザクション署名に失敗: {}",
                message.payload.error_message
            )))
        }
    }

    /// 署名要求を処理
    fn handle_sign_request(&self, addr_from: String, address: String, transaction: Transaction) -> Result<()> {
        info!(
            "署名要求を受信: {} ウォレット: {}",
            addr_from, address
        );

        let wallet_config = crate::config::WalletConfig {
            data_dir: format!("{}/wallets", self.inner.lock().unwrap().utxo.blockchain.config.data_dir),
            default_key_type: CryptoType::FNDSA,
        };
        
        let wallets = Wallets::new(wallet_config)?;
        let wallet_address = Address(address);
        
        let wallet = match wallets.get_wallet(&wallet_address) {
            Some(w) => w,
            None => {
                let response = SignResponsePayload {
                    transaction: transaction.clone(),
                    success: false,
                    error_message: format!("ウォレットが見つかりません: {}", wallet_address),
                };
                
                let message = NetworkMessage {
                    message_type: MessageType::SignResponse,
                    addr_from: self.node_address.clone(),
                    payload: response,
                };
                
                let data = serialize(&message)?;
                self.send_data(&addr_from, &data)?;
                return Ok(());
            }
        };

        let mut tx = transaction.clone();
        let crypto = FnDsaCrypto;

        match self.inner.lock().unwrap().utxo.blockchain.sign_transacton(
            &mut tx,
            &wallet.secret_key,
            &crypto,
        ) {
            Ok(_) => {
                // 署名成功
                let response = SignResponsePayload {
                    transaction: tx,
                    success: true,
                    error_message: String::new(),
                };
                
                let message = NetworkMessage {
                    message_type: MessageType::SignResponse,
                    addr_from: self.node_address.clone(),
                    payload: response,
                };
                
                let data = serialize(&message)?;
                self.send_data(&addr_from, &data)?;
            }
            Err(e) => {
                // 署名失敗
                let response = SignResponsePayload {
                    transaction: transaction,
                    success: false,
                    error_message: format!("署名エラー: {}", e),
                };
                
                let message = NetworkMessage {
                    message_type: MessageType::SignResponse,
                    addr_from: self.node_address.clone(),
                    payload: response,
                };
                
                let data = serialize(&message)?;
                self.send_data(&addr_from, &data)?;
            }
        }

        Ok(())
    }

    /// 接続を処理
    fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        info!("接続を受付: {:?}", stream.peer_addr()?);

        let mut buffer = vec![0; 4096];
        let count = stream.read_to_end(&mut buffer)?;
        buffer.truncate(count);

        info!("リクエストを受信: {} バイト", count);

        if count == 0 {
            return Ok(());
        }

        let result: bincode::Result<NetworkMessage<serde_json::Value>> = deserialize(&buffer);
        
        if let Ok(message) = result {
            match message.message_type {
                MessageType::Addr => {
                    let msg: NetworkMessage<AddrMessagePayload> = deserialize(&buffer)?;
                    self.handle_addr(msg.payload.addresses)?;
                },
                MessageType::Block => {
                    let msg: NetworkMessage<BlockMessagePayload> = deserialize(&buffer)?;
                    self.handle_block(msg.addr_from, msg.payload.block)?;
                },
                MessageType::Inv => {
                    let msg: NetworkMessage<InvMessagePayload> = deserialize(&buffer)?;
                    self.handle_inv(msg.addr_from, msg.payload.kind, msg.payload.items)?;
                },
                MessageType::GetBlock => {
                    let msg: NetworkMessage<GetBlocksMessagePayload> = deserialize(&buffer)?;
                    self.handle_get_blocks(msg.addr_from)?;
                },
                MessageType::GetData => {
                    let msg: NetworkMessage<GetDataMessagePayload> = deserialize(&buffer)?;
                    self.handle_get_data(msg.addr_from, msg.payload.kind, msg.payload.id)?;
                },
                MessageType::Tx => {
                    let msg: NetworkMessage<TxMessagePayload> = deserialize(&buffer)?;
                    self.handle_tx(msg.addr_from, msg.payload.transaction)?;
                },
                MessageType::Version => {
                    let msg: NetworkMessage<VersionMessagePayload> = deserialize(&buffer)?;
                    self.handle_version(msg.addr_from, msg.payload)?;
                },
                MessageType::SignRequest => {
                    let msg: NetworkMessage<SignRequestPayload> = deserialize(&buffer)?;
                    self.handle_sign_request(msg.addr_from, msg.payload.address, msg.payload.transaction)?;
                },
                MessageType::SignResponse => {
                    // 署名応答はここでは直接処理しない（別の処理フローで使用）
                },
            }
        } else {
            error!("メッセージのデシリアライズに失敗: {:?}", result.err());
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::blockchain::blockchain::*;
    // use crate::crypto::types::CryptoType;
    use crate::config::{BlockchainConfig, NetworkConfig};
    
    #[test]
    fn test_message_serialization() -> Result<()> {
        let config = NetworkConfig::default();
        
        // バージョンメッセージのテスト
        let version_payload = VersionMessagePayload {
            version: 1,
            best_height: 10,
        };
        
        let version_message = NetworkMessage {
            message_type: MessageType::Version,
            addr_from: "localhost:7878".to_string(),
            payload: version_payload.clone(),
        };
        
        let serialized = serialize(&version_message)?;
        let deserialized: NetworkMessage<VersionMessagePayload> = deserialize(&serialized)?;
        
        assert_eq!(deserialized.message_type, MessageType::Version);
        assert_eq!(deserialized.addr_from, "localhost:7878");
        assert_eq!(deserialized.payload.version, version_payload.version);
        assert_eq!(deserialized.payload.best_height, version_payload.best_height);
        
        Ok(())
    }
    
    #[test]
    fn test_node_management() -> Result<()> {
        let blockchain_config = BlockchainConfig::default();
        let network_config = NetworkConfig::default();
        
        // テスト用のアドレスを作成
        let mining_address = Address("test_address".to_string());
        
        // ブロックチェーンとUTXOセットを作成
        let blockchain = Blockchain::create_blockchain(mining_address.clone(), blockchain_config.clone())?;
        let utxo_set = UTXOSet { blockchain };
        
        // サーバーを作成
        let server = Server::new(
            "localhost",
            "7878",
            &mining_address,
            None,
            utxo_set,
            network_config,
        )?;
        
        // ノード管理機能のテスト
        assert_eq!(server.get_known_nodes().len(), 0);
        
        // ノードを追加
        server.add_nodes("localhost:1234");
        assert_eq!(server.get_known_nodes().len(), 1);
        assert!(server.node_is_known("localhost:1234"));
        
        // ノードを削除
        server.remove_node("localhost:1234");
        assert_eq!(server.get_known_nodes().len(), 0);
        assert!(!server.node_is_known("localhost:1234"));
        
        Ok(())
    }
    
    #[test]
    fn test_mempool_management() -> Result<()> {
        let blockchain_config = BlockchainConfig::default();
        let network_config = NetworkConfig::default();
        
        // テスト用のアドレスを作成
        let mining_address = Address("test_address".to_string());
        
        // ブロックチェーンとUTXOセットを作成
        let blockchain = Blockchain::create_blockchain(mining_address.clone(), blockchain_config.clone())?;
        let utxo_set = UTXOSet { blockchain };
        
        // サーバーを作成
        let server = Server::new(
            "localhost",
            "7879",
            &mining_address,
            None,
            utxo_set,
            network_config,
        )?;
        
        // メモリプール管理機能のテスト
        assert_eq!(server.get_mempool().len(), 0);
        
        // コインベーストランザクションを作成してメモリプールに追加
        let tx = Transaction::new_coinbase(
            mining_address.clone(),
            "test_data".to_string(),
            &blockchain_config,
        )?;
        
        server.insert_mempool(tx.clone());
        
        // メモリプールに追加されたことを確認
        assert_eq!(server.get_mempool().len(), 1);
        assert!(server.get_mempool_tx(&tx.id).is_some());
        
        // メモリプールをクリア
        server.clear_mempool();
        assert_eq!(server.get_mempool().len(), 0);
        
        Ok(())
    }
}
