(module
  (func (export "main") (result i32)
    i32.const 42)
  (func (export "add") (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add)
  (memory (export "memory") 1)
)
