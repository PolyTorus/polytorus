pub mod block;
// Legacy blockchain implementation removed in Phase 4
pub mod types;
// Legacy UTXO set removed in Phase 4 - replaced by ModularStorage
// pub mod utxoset;

#[cfg(test)]
pub mod difficulty_tests;
