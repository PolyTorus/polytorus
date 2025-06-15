//! 基本的なKani検証テスト

#[cfg(kani)]
#[kani::proof]
pub fn test_basic_verification() {
    let x = 5u32;
    let y = 10u32;
    
    assert!(x < y);
    assert!(x + y == 15);
}
