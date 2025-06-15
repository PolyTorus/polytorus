use crate::config::DataContext;
use std::path::PathBuf;
use uuid::Uuid;

pub fn create_test_context() -> DataContext {
    let test_id = Uuid::new_v4();
    let base_dir = PathBuf::from(format!("test_data_{}", test_id));
    DataContext::new(base_dir)
}

pub fn cleanup_test_context(context: &DataContext) {
    std::fs::remove_dir_all(&context.data_dir).ok();
}

// RAII guard for automatic cleanup
pub struct TestContextGuard {
    context: DataContext,
}

impl TestContextGuard {
    pub fn new(context: DataContext) -> Self {
        Self { context }
    }

    pub fn context(&self) -> &DataContext {
        &self.context
    }
}

impl Drop for TestContextGuard {
    fn drop(&mut self) {
        cleanup_test_context(&self.context);
    }
}

#[cfg(test)]
mod tests {
    use super::*;    #[test]
    fn test_context_creation() {
        let context = create_test_context();
        assert!(context.data_dir.to_string_lossy().contains("test_data"));
        cleanup_test_context(&context);
    }
}
