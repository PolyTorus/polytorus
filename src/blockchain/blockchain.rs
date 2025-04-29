//! Blockchain

use crate::blockchain::block::*;
use crate::crypto::traits::CryptoProvider;
use crate::crypto::transaction::*;
use crate::Result;
use bincode::{deserialize, serialize};
use failure::format_err;
use sled;
use std::collections::HashMap;
use std::time::SystemTime;

const GENESIS_COINBASE_DATA: &str =
    "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

/// Blockchain implements interactions with a DB
#[derive(Debug)]
pub struct Blockchain {
    pub tip: String,
    pub db: sled::Db,
}

/// BlockchainIterator is used to iterate over blockchain blocks
pub struct BlockchainIterator<'a> {
    current_hash: String,
    bc: &'a Blockchain,
}

impl Blockchain {
    /// NewBlockchain creates a new Blockchain db
    pub fn new() -> Result<Blockchain> {
        info!("open blockchain");

        let db = sled::open("data/blocks")?;
        let hash = match db.get("LAST")? {
            Some(l) => l.to_vec(),
            None => Vec::new(),
        };
        info!("Found block database");
        let lasthash = if hash.is_empty() {
            String::new()
        } else {
            String::from_utf8(hash.to_vec())?
        };
        Ok(Blockchain { tip: lasthash, db })
    }

    /// CreateBlockchain creates a new blockchain DB
    pub fn create_blockchain(address: String) -> Result<Blockchain> {
        info!("Creating new blockchain");

        std::fs::remove_dir_all("data/blocks").ok();
        let db = sled::open("data/blocks")?;
        debug!("Creating new block database");
        let cbtx = Transaction::new_coinbase(address, String::from(GENESIS_COINBASE_DATA))?;
        let genesis: Block = Block::new_genesis_block(cbtx);
        db.insert(genesis.get_hash(), serialize(&genesis)?)?;
        db.insert("LAST", genesis.get_hash().as_bytes())?;
        let bc = Blockchain {
            tip: genesis.get_hash(),
            db,
        };
        bc.db.flush()?;
        Ok(bc)
    }

    /// MineBlock mines a new block with the provided transactions
    pub fn mine_block(&mut self, transactions: Vec<Transaction>) -> Result<Block> {
        info!("mine a new block");

        for tx in &transactions {
            if !self.verify_transacton(tx)? {
                return Err(format_err!("ERROR: Invalid transaction"));
            }
        }

        let lasthash = self.db.get("LAST")?.unwrap();
        let prev_hash = String::from_utf8(lasthash.to_vec())?;
        let prev_block = self.get_block(&prev_hash)?;
        let current_timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let new_difficulty = Block::adjust_difficulty(&prev_block, current_timestamp);

        let newblock = Block::new_block(
            transactions,
            prev_hash,
            self.get_best_height()? + 1,
            new_difficulty,
        )?;
        self.db.insert(newblock.get_hash(), serialize(&newblock)?)?;
        self.db.insert("LAST", newblock.get_hash().as_bytes())?;
        self.db.flush()?;

        self.tip = newblock.get_hash();
        Ok(newblock)
    }

    /// Iterator returns a BlockchainIterat
    pub fn iter(&self) -> BlockchainIterator {
        BlockchainIterator {
            current_hash: self.tip.clone(),
            bc: self,
        }
    }

    /// FindUTXO finds and returns all unspent transaction outputs
    pub fn find_UTXO(&self) -> HashMap<String, TXOutputs> {
        let mut utxos: HashMap<String, TXOutputs> = HashMap::new();
        let mut spend_txos: HashMap<String, Vec<i32>> = HashMap::new();

        for block in self.iter() {
            for tx in block.get_transaction() {
                for index in 0..tx.vout.len() {
                    if let Some(ids) = spend_txos.get(&tx.id) {
                        if ids.contains(&(index as i32)) {
                            continue;
                        }
                    }

                    match utxos.get_mut(&tx.id) {
                        Some(v) => {
                            v.outputs.push(tx.vout[index].clone());
                        }
                        None => {
                            utxos.insert(
                                tx.id.clone(),
                                TXOutputs {
                                    outputs: vec![tx.vout[index].clone()],
                                },
                            );
                        }
                    }
                }

                if !tx.is_coinbase() {
                    for i in &tx.vin {
                        match spend_txos.get_mut(&i.txid) {
                            Some(v) => {
                                v.push(i.vout);
                            }
                            None => {
                                spend_txos.insert(i.txid.clone(), vec![i.vout]);
                            }
                        }
                    }
                }
            }
        }

        utxos
    }

    /// FindTransaction finds a transaction by its ID
    pub fn find_transacton(&self, id: &str) -> Result<Transaction> {
        for b in self.iter() {
            for tx in b.get_transaction() {
                if tx.id == id {
                    return Ok(tx.clone());
                }
            }
        }
        Err(format_err!("Transaction is not found"))
    }

    fn get_prev_TXs(&self, tx: &Transaction) -> Result<HashMap<String, Transaction>> {
        let mut prev_TXs = HashMap::new();
        for vin in &tx.vin {
            let prev_TX = self.find_transacton(&vin.txid)?;
            prev_TXs.insert(prev_TX.id.clone(), prev_TX);
        }
        Ok(prev_TXs)
    }

    /// SignTransaction signs inputs of a Transaction
    pub fn sign_transacton(
        &self,
        tx: &mut Transaction,
        private_key: &[u8],
        crypto: &dyn CryptoProvider,
    ) -> Result<()> {
        let prev_TXs = self.get_prev_TXs(tx)?;
        tx.sign(private_key, prev_TXs, crypto)?;
        Ok(())
    }

    /// VerifyTransaction verifies transaction input signatures
    pub fn verify_transacton(&self, tx: &Transaction) -> Result<bool> {
        if tx.is_coinbase() {
            return Ok(true);
        }
        let prev_TXs = self.get_prev_TXs(tx)?;
        tx.verify(prev_TXs)
    }

    /// AddBlock saves the block into the blockchain
    pub fn add_block(&mut self, block: Block) -> Result<()> {
        let data = serialize(&block)?;
        if let Some(_) = self.db.get(block.get_hash())? {
            return Ok(());
        }
        self.db.insert(block.get_hash(), data)?;

        let lastheight = self.get_best_height()?;
        if block.get_height() > lastheight {
            self.db.insert("LAST", block.get_hash().as_bytes())?;
            self.tip = block.get_hash();
            self.db.flush()?;
        }
        Ok(())
    }

    // GetBlock finds a block by its hash and returns it
    pub fn get_block(&self, block_hash: &str) -> Result<Block> {
        let data = self.db.get(block_hash)?.unwrap();
        let block = deserialize(&data)?;
        Ok(block)
    }

    /// GetBestHeight returns the height of the latest block
    pub fn get_best_height(&self) -> Result<i32> {
        let lasthash = if let Some(h) = self.db.get("LAST")? {
            h
        } else {
            return Ok(-1);
        };
        let last_data = self.db.get(lasthash)?.unwrap();
        let last_block: Block = deserialize(&last_data)?;
        Ok(last_block.get_height())
    }

    /// GetBlockHashes returns a list of hashes of all the blocks in the chain
    pub fn get_block_hashs(&self) -> Vec<String> {
        let mut list = Vec::new();
        for b in self.iter() {
            list.push(b.get_hash());
        }
        list
    }
}

impl Iterator for BlockchainIterator<'_> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(encoded_block) = self.bc.db.get(&self.current_hash) {
            return match encoded_block {
                Some(b) => {
                    if let Ok(block) = deserialize::<Block>(&b) {
                        self.current_hash = block.get_prev_hash();
                        Some(block)
                    } else {
                        None
                    }
                }
                None => None,
            };
        }
        None
    }
}

mod tests {
    use super::*;
    use crate::crypto::transaction::Transaction;
    use crate::crypto::fndsa::FnDsaCrypto;
    use crate::crypto::types::EncryptionType;
    use crate::crypto::wallets::{Wallets, Wallet}; 
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::hash::{Hash, Hasher};
    use std::thread;
    use std::fs;
    use std::path::Path;
    use fn_dsa::{sign_key_size, vrfy_key_size, KeyPairGenerator, KeyPairGeneratorStandard, FN_DSA_LOGN_512};
    use rand_core::OsRng;

    // 以下の関数はwallets.rsのものをインポートするのではなく、テストモジュール内で直接定義

    // 安全なファイル削除 - リトライ機能付き
fn safe_remove_dir(path: &str, max_retries: usize) -> Result<()> {
    let path = Path::new(path);
    if !path.exists() {
        return Ok(());
    }
    
    for attempt in 0..max_retries {
        match fs::remove_dir_all(path) {
            Ok(_) => return Ok(()),
            Err(e) => {
                if attempt == max_retries - 1 {
                    return Err(format_err!("Failed to remove directory after {} attempts: {}", max_retries, e));
                }
                // エラーが発生した場合、少し待機してリトライ
                thread::sleep(std::time::Duration::from_millis(50 * (attempt as u64 + 1)));
            }
        }
    }
    Ok(())
}

// テストごとに完全に独立したデータベースパスを生成（プロセスIDとタイムスタンプを含む）
fn unique_test_path(prefix: &str) -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::hash::{Hash, Hasher};
    use std::process;
    
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    
    // スレッドIDとプロセスIDの組み合わせで一意性を確保
    let thread_id = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        thread::current().id().hash(&mut hasher);
        hasher.finish()
    };
    
    // 現在のタイムスタンプも加えて、同一プロセス内でも確実に一意になるようにする
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    
    format!("data/test_{}_{}_{}_{}_{}",
        prefix,
        process::id(),      // プロセスID
        thread_id,          // スレッドID
        timestamp % 10000,  // タイムスタンプ（短縮版）
        id                  // カウンター
    )
}

    // テスト用に一意のデータベースパスを生成する
    fn temp_blockchain_path() -> String {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        
        // スレッドIDをハッシュ化して一意にする
        let thread_id = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            thread::current().id().hash(&mut hasher);
            hasher.finish()
        };
        
        format!("data/test_blocks_{}_{}", thread_id, id)
    }

    // テスト用に一意のウォレットパスを生成
    fn temp_wallet_path() -> String {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        
        // スレッドIDをハッシュ化して一意にする
        let thread_id = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            thread::current().id().hash(&mut hasher);
            hasher.finish()
        };
        
        format!("data/test_wallets_{}_{}", thread_id, id)
    }

    // テスト用のBlockchain作成関数
    fn new_test_blockchain(path: &str) -> Result<Blockchain> {
        info!("open blockchain in {}", path);

        let db_config = sled::Config::new()
            .path(path)
            .mode(sled::Mode::HighThroughput)
            .flush_every_ms(Some(1000))
            .use_compression(false) // 速度重視
            .cache_capacity(10_485_760) // 10MB
            .temporary(true); // テスト用なので一時的なデータベースとして扱う
        
        let db = db_config.open()?;
        
        let hash = match db.get("LAST")? {
            Some(l) => l.to_vec(),
            None => Vec::new(),
        };
        
        let lasthash = if hash.is_empty() {
            String::new()
        } else {
            String::from_utf8(hash.to_vec())?
        };
        
        Ok(Blockchain { tip: lasthash, db })
    }

    // テスト用のブロックチェーン生成
    fn create_test_blockchain(address: String, path: &str) -> Result<Blockchain> {
        info!("Creating new blockchain in {}", path);

        std::fs::remove_dir_all(path).ok();
        
        let db_config = sled::Config::new()
            .path(path)
            .mode(sled::Mode::HighThroughput)
            .flush_every_ms(Some(1000))
            .use_compression(false) // 速度重視
            .cache_capacity(10_485_760) // 10MB
            .temporary(true); // テスト用なので一時的なデータベースとして扱う
        
        let db = db_config.open()?;
        
        debug!("Creating new block database");
        let cbtx = Transaction::new_coinbase(address, String::from(GENESIS_COINBASE_DATA))?;
        let genesis: Block = Block::new_genesis_block(cbtx);
        db.insert(genesis.get_hash(), serialize(&genesis)?)?;
        db.insert("LAST", genesis.get_hash().as_bytes())?;
        
        let bc = Blockchain {
            tip: genesis.get_hash(),
            db,
        };
        
        bc.db.flush()?;
        Ok(bc)
    }

    // テスト用ウォレット管理
    fn new_test_wallets(path: &str) -> Result<Wallets> {
        let mut wlt = Wallets {
            wallets: HashMap::<String, Wallet>::new(),
        };
        
        // パスが存在する場合は読み込み
        let db_config = sled::Config::new()
            .path(path)
            .mode(sled::Mode::HighThroughput);
        
        let db = db_config.open()?;

        for item in db.iter() {
            let i = item?;
            let address = String::from_utf8(i.0.to_vec())?;
            let wallet = deserialize(&i.1)?;
            wlt.wallets.insert(address, wallet);
        }
        
        Ok(wlt)
    }

    // テスト用のウォレット保存機能
    fn save_test_wallets(wallets: &Wallets, path: &str) -> Result<()> {
        let db_config = sled::Config::new()
            .path(path)
            .mode(sled::Mode::HighThroughput);
        
        let db = db_config.open()?;

        for (address, wallet) in &wallets.wallets {
            let data = serialize(wallet)?;
            db.insert(address, data)?;
        }

        db.flush()?;
        Ok(())
    }

    // 固定のモックウォレットを作成（テスト用）
    fn create_fixed_test_wallet() -> (Wallet, String) {
        // FNDSAのキーペアを生成
        let mut kg = KeyPairGeneratorStandard::default();
        let mut sign_key = [0u8; sign_key_size(FN_DSA_LOGN_512)];
        let mut vrfy_key = [0u8; vrfy_key_size(FN_DSA_LOGN_512)];
        kg.keygen(FN_DSA_LOGN_512, &mut OsRng, &mut sign_key, &mut vrfy_key);
        
        let wallet = Wallet {
            secret_key: sign_key.to_vec(),
            public_key: vrfy_key.to_vec(),
        };
        
        let address = wallet.get_address();
        (wallet, address)
    }

    // テスト設定のセットアップとクリーンアップのためのヘルパー構造体
    struct TestBlockchain {
        blockchain: Blockchain,
        path: String,
    }

    impl TestBlockchain {
        fn new() -> Result<Self> {
            let path = temp_blockchain_path();
            
            // 固定のテストウォレットを使用
            let (_, address) = create_fixed_test_wallet();
            
            let blockchain = create_test_blockchain(address, &path)?;
            Ok(Self { blockchain, path })
        }
    }

    impl Drop for TestBlockchain {
        fn drop(&mut self) {
            // テスト終了時にディレクトリを削除
            if Path::new(&self.path).exists() {
                fs::remove_dir_all(&self.path).ok();
            }
        }
    }

    #[test]
    fn test_new_blockchain() -> Result<()> {
        // より一意なパスを使用
    let test_path = unique_test_path("blockchain");
    
    // より安全なクリーンアップ
    safe_remove_dir(&test_path, 3)?;
    
    // 新しいブロックチェーンを作成
    let (_, address) = create_fixed_test_wallet();
    
    let bc = create_test_blockchain(address.clone(), &test_path)?;
    assert_eq!(bc.get_best_height()?, 0);
    
    // 明示的にフラッシュしてからドロップ
    bc.db.flush()?;
    drop(bc);
    thread::sleep(std::time::Duration::from_millis(100));
    
    // 再読み込み
    let loaded_bc = new_test_blockchain(&test_path)?;
    assert_eq!(loaded_bc.get_best_height()?, 0);
    
    // 明示的にクローズ
    loaded_bc.db.flush()?;
    drop(loaded_bc);
    thread::sleep(std::time::Duration::from_millis(100));
    
    // クリーンアップ
    safe_remove_dir(&test_path, 3)?;
    
    Ok(())
    }

    #[test]
    fn test_mine_block() -> Result<()> {
        // テスト設定のセットアップ
        let mut test = TestBlockchain::new()?;
        let bc = &mut test.blockchain;
        
        // 新しいブロックを採掘（有効なアドレスを使用）
        let (_, address) = create_fixed_test_wallet();
        let cbtx = Transaction::new_coinbase(address, String::from("reward!"))?;
        let new_block = bc.mine_block(vec![cbtx])?;
        
        // 採掘結果を検証
        assert_eq!(bc.get_best_height()?, 1);
        assert_eq!(new_block.get_height(), 1);
        
        // テスト設定はドロップ時に自動的にクリーンアップされる
        Ok(())
    }

    #[test]
    fn test_blockchain_iterator() -> Result<()> {
        // テスト設定のセットアップ
        let mut test = TestBlockchain::new()?;
        let bc = &mut test.blockchain;
        
        // いくつかのブロックを採掘（有効なアドレスを使用）
        for i in 0..3 {
            let (_, address) = create_fixed_test_wallet();
            let cbtx = Transaction::new_coinbase(
                address, 
                format!("reward_{}", i)
            )?;
            bc.mine_block(vec![cbtx])?;
        }
        
        // イテレータをテスト
        let mut heights = Vec::new();
        for block in bc.iter() {
            heights.push(block.get_height());
        }
        
        // 高さの検証（イテレータは逆順）
        assert_eq!(heights, vec![3, 2, 1, 0]);
        
        Ok(())
    }

    #[test]
    fn test_find_transaction() -> Result<()> {
        // テスト設定のセットアップ
        let mut test = TestBlockchain::new()?;
        let bc = &mut test.blockchain;
        
        // トランザクションを含むブロックを採掘
        let (_, address) = create_fixed_test_wallet();
        let cbtx = Transaction::new_coinbase(address, String::from("reward!"))?;
        let tx_id = cbtx.id.clone();
        bc.mine_block(vec![cbtx])?;
        
        // トランザクションを検索
        let found_tx = bc.find_transacton(&tx_id)?;
        assert_eq!(found_tx.id, tx_id);
        
        // 存在しないトランザクションを検索
        let result = bc.find_transacton("nonexistent_id");
        assert!(result.is_err());
        
        Ok(())
    }

    #[test]
    fn test_verify_transaction() -> Result<()> {
        // テスト設定のセットアップ
        let test = TestBlockchain::new()?;
        let bc = &test.blockchain;
        
        // コインベーストランザクションは常に有効
        let (_, address) = create_fixed_test_wallet();
        let cbtx = Transaction::new_coinbase(address, String::from("reward!"))?;
        assert!(bc.verify_transacton(&cbtx)?);
        
        // 無効なトランザクションの検証
        // 実際のトランザクション検証ロジックをテスト
        let mut invalid_tx = Transaction {
            id: "invalid_tx".to_string(),
            vin: vec![TXInput {
                txid: "nonexistent_id".to_string(),
                vout: 0,
                signature: Vec::new(),
                pub_key: Vec::new(),
            }],
            vout: Vec::new(),
        };
        // トランザクションIDを設定
        invalid_tx.id = invalid_tx.hash()?;
        
        // 無効なトランザクションの検証は失敗するはず
        assert!(bc.verify_transacton(&invalid_tx).is_err());
        
        Ok(())
    }

    #[test]
    fn test_sign_transaction() -> Result<()> {
        // テスト設定のセットアップ
        let test_path = temp_blockchain_path();
        
        // クリーンな状態でテスト開始
        if Path::new(&test_path).exists() {
            fs::remove_dir_all(&test_path)?;
            thread::sleep(std::time::Duration::from_millis(100));
        }
        
        // 固定のテストウォレットを使用
        let (wallet, address) = create_fixed_test_wallet();
        
        // まずブロックチェーンを作成
        let mut bc = create_test_blockchain(address.clone(), &test_path)?;
        
        // トランザクションを作成して先にブロックに追加する
        // このステップが重要 - sign_transaction を呼び出す前にトランザクションが
        // ブロックチェーンに存在している必要がある
        let coinbase_tx = Transaction::new_coinbase(address.clone(), String::from("reward!"))?;
        let mut tx = Transaction {
            id: String::new(),
            vin: vec![TXInput {
                txid: coinbase_tx.id.clone(), // コインベーストランザクションを参照
                vout: 0,
                signature: Vec::new(),
                pub_key: wallet.public_key.clone(),
            }],
            vout: vec![TXOutput::new(5, address.clone())?], // 適当な出力
        };
        
        // トランザクションIDを設定
        tx.id = tx.hash()?;
        
        // コインベーストランザクションをブロックに追加
        bc.mine_block(vec![coinbase_tx])?;
        
        // 暗号化プロバイダー
        let crypto = FnDsaCrypto;
        
        // トランザクションに署名
        bc.sign_transacton(&mut tx, &wallet.secret_key, &crypto)?;
        
        // 署名されたことを確認（署名フィールドが空でないこと）
        assert!(!tx.vin[0].signature.is_empty());
        
        // 署名の検証
        assert!(bc.verify_transacton(&tx)?);
        
        // データベースを閉じて確実に解放
        drop(bc);
        thread::sleep(std::time::Duration::from_millis(100));
        
        // クリーンアップ
        fs::remove_dir_all(&test_path)?;
        
        Ok(())
    }

    #[test]
    fn test_add_block() -> Result<()> {
        // 二つの独立したブロックチェーンを作成
        let test_path1 = temp_blockchain_path();
        let test_path2 = temp_blockchain_path();
        
        // 固定のテストウォレットを使用
        let (_, address) = create_fixed_test_wallet();
        
        let mut bc1 = create_test_blockchain(address.clone(), &test_path1)?;
        let mut bc2 = create_test_blockchain(address.clone(), &test_path2)?;
        
        // bc1にブロックを追加
        let cbtx = Transaction::new_coinbase(address.clone(), String::from("reward!"))?;
        let block = bc1.mine_block(vec![cbtx])?;
        
        // bc1の高さを確認
        assert_eq!(bc1.get_best_height()?, 1);
        
        // bc2にbc1のブロックを追加
        bc2.add_block(block.clone())?;
        
        // bc2の高さも1になったことを確認
        assert_eq!(bc2.get_best_height()?, 1);
        
        // bc1とbc2のブロックが同じであることを確認
        let bc1_block = bc1.get_block(&block.get_hash())?;
        let bc2_block = bc2.get_block(&block.get_hash())?;
        
        assert_eq!(bc1_block.get_hash(), bc2_block.get_hash());
        
        // クリーンアップ
        fs::remove_dir_all(&test_path1).ok();
        fs::remove_dir_all(&test_path2).ok();
        
        Ok(())
    }
}
