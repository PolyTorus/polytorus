// src/types.rs
// 共通型定義とモナド関連の基本型

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// アプリケーション全体で使用する Result 型
pub type Result<T> = std::result::Result<T, failure::Error>;

/// モナド型特性 - 関数合成の基盤
pub trait Monad: Sized {
    type Item;
    
    /// 値をモナドにリフト
    fn unit(value: Self::Item) -> Self;
    
    /// モナド間の関数合成（bind操作）
    fn bind<B, F>(self, f: F) -> B
    where
        F: FnOnce(Self::Item) -> B + 'static;
    
    /// モナド内の値を変換（map操作）
    fn map<F>(self, f: F) -> Self
    where
        F: FnOnce(Self::Item) -> Self::Item + 'static;
}

/// ブロックチェーン操作のためのIO モナド
pub struct BlockchainIO<T, E> {
    run: Box<dyn FnOnce() -> std::result::Result<T, E>>,
    _phantom: PhantomData<(T, E)>,
}

impl<T, E> BlockchainIO<T, E> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> std::result::Result<T, E> + 'static,
    {
        BlockchainIO {
            run: Box::new(f),
            _phantom: PhantomData,
        }
    }
    
    pub fn execute(self) -> std::result::Result<T, E> {
        (self.run)()
    }
}

impl<T: 'static, E: 'static> Monad for BlockchainIO<T, E> {
    type Item = T;
    
    fn unit(value: Self::Item) -> Self {
        BlockchainIO::new(move || Ok(value))
    }
    
    fn bind<B, F>(self, f: F) -> B
    where
        F: FnOnce(Self::Item) -> B + 'static,
    {
        // 具体的な実装はここでは難しいため、ダミー値を返す
        // 実際の実装ではエラーが発生します
        panic!("Cannot implement generic bind directly. Use and_then instead.")
    }
    
    fn map<F>(self, f: F) -> Self
    where
        F: FnOnce(Self::Item) -> Self::Item + 'static,
    {
        let new_run = || {
            let result = (self.run)()?;
            Ok(f(result))
        };
        
        BlockchainIO::new(new_run)
    }
}

// BlockchainIOに対する実用的な拡張メソッド
impl<T: 'static, E: 'static> BlockchainIO<T, E> {
    /// モナドの結合操作（bindに相当）
    pub fn and_then<U, F>(self, f: F) -> BlockchainIO<U, E>
    where
        F: FnOnce(T) -> BlockchainIO<U, E> + 'static,
        U: 'static,
    {
        BlockchainIO::new(move || {
            let result = (self.run)()?;
            f(result).execute()
        })
    }
    
    /// 値の変換操作（拡張版map）
    pub fn map_to<U, F>(self, f: F) -> BlockchainIO<U, E>
    where
        F: FnOnce(T) -> U + 'static,
        U: 'static,
    {
        BlockchainIO::new(move || {
            let result = (self.run)()?;
            Ok(f(result))
        })
    }
}

/// 暗号化アルゴリズムを表す型レベルの値
pub enum EncryptionType {
    ECDSA,
    FNDSA,
}

/// 関数合成ヘルパー
pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |a| g(f(a))
}

/// 部分適用ヘルパー
pub fn partial<A, B, C, F>(f: F, a: A) -> impl Fn(B) -> C
where
    F: Fn(A, B) -> C,
    A: Clone,
{
    move |b| f(a.clone(), b)
}

/// ファンネル関数: 値を取り、その値に対して複数の関数を適用して結果をタプルで返す
pub fn fanout<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> (B, C)
where
    F: Fn(A) -> B,
    G: Fn(A) -> C,
    A: Clone,
{
    move |a| (f(a.clone()), g(a))
}
