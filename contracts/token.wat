;; Simple Token Contract - demonstrates complex state management
(module
  ;; Import host functions
  (import "env" "storage_get" (func $storage_get (param i32 i32) (result i32)))
  (import "env" "storage_set" (func $storage_set (param i32 i32 i32 i32)))
  (import "env" "log" (func $log (param i32 i32)))
  (import "env" "get_caller" (func $get_caller (result i32)))
  (import "env" "get_value" (func $get_value (result i64)))

  ;; Memory for contract operations
  (memory 2)
  
  ;; Global constants for storage keys
  (global $total_supply_key_ptr i32 (i32.const 0))
  (global $total_supply_key_len i32 (i32.const 12))
  (global $balance_prefix_ptr i32 (i32.const 16))
  (global $balance_prefix_len i32 (i32.const 8))
  
  ;; String constants
  (data (i32.const 0) "total_supply")
  (data (i32.const 16) "balance_")
  (data (i32.const 32) "Token initialized with supply: ")
  (data (i32.const 64) "Transfer successful")
  (data (i32.const 82) "Transfer failed: insufficient balance")
  (data (i32.const 120) "Mint successful")
  (data (i32.const 136) "Burn successful")

  ;; Initialize the token contract with total supply
  (func (export "init") (param $initial_supply i32) (result i32)
    (local $caller i32)
    
    ;; Get the caller (contract deployer)
    (local.set $caller (call $get_caller))
    
    ;; Set total supply
    (call $set_total_supply (local.get $initial_supply))
    
    ;; Give all initial tokens to the deployer
    (call $set_balance (local.get $caller) (local.get $initial_supply))
    
    ;; Log initialization
    (call $log (i32.const 32) (i32.const 31))
    
    (i32.const 1) ;; return success
  )

  ;; Get total supply
  (func (export "total_supply") (result i32)
    (call $get_total_supply)
  )

  ;; Get balance of an address
  (func (export "balance_of") (param $address i32) (result i32)
    (call $get_balance (local.get $address))
  )

  ;; Transfer tokens from caller to recipient
  (func (export "transfer") (param $to i32) (param $amount i32) (result i32)
    (local $caller i32)
    (local $caller_balance i32)
    (local $recipient_balance i32)
    
    ;; Get caller
    (local.set $caller (call $get_caller))
    
    ;; Check if caller has enough balance
    (local.set $caller_balance (call $get_balance (local.get $caller)))
    
    (if (i32.lt_u (local.get $caller_balance) (local.get $amount))
      (then
        ;; Insufficient balance
        (call $log (i32.const 82) (i32.const 37))
        (return (i32.const 0))
      )
    )
    
    ;; Get recipient balance
    (local.set $recipient_balance (call $get_balance (local.get $to)))
    
    ;; Update balances
    (call $set_balance 
      (local.get $caller) 
      (i32.sub (local.get $caller_balance) (local.get $amount)))
    
    (call $set_balance 
      (local.get $to) 
      (i32.add (local.get $recipient_balance) (local.get $amount)))
    
    ;; Log success
    (call $log (i32.const 64) (i32.const 18))
    
    (i32.const 1) ;; return success
  )

  ;; Mint new tokens (only for demonstration)
  (func (export "mint") (param $to i32) (param $amount i32) (result i32)
    (local $current_supply i32)
    (local $recipient_balance i32)
    
    ;; Get current supply and recipient balance
    (local.set $current_supply (call $get_total_supply))
    (local.set $recipient_balance (call $get_balance (local.get $to)))
    
    ;; Update total supply
    (call $set_total_supply (i32.add (local.get $current_supply) (local.get $amount)))
    
    ;; Add tokens to recipient
    (call $set_balance 
      (local.get $to) 
      (i32.add (local.get $recipient_balance) (local.get $amount)))
    
    ;; Log success
    (call $log (i32.const 120) (i32.const 15))
    
    (i32.const 1)
  )

  ;; Burn tokens from caller's balance
  (func (export "burn") (param $amount i32) (result i32)
    (local $caller i32)
    (local $caller_balance i32)
    (local $current_supply i32)
    
    ;; Get caller and balance
    (local.set $caller (call $get_caller))
    (local.set $caller_balance (call $get_balance (local.get $caller)))
    
    ;; Check sufficient balance
    (if (i32.lt_u (local.get $caller_balance) (local.get $amount))
      (then
        (call $log (i32.const 82) (i32.const 37))
        (return (i32.const 0))
      )
    )
    
    ;; Get current supply
    (local.set $current_supply (call $get_total_supply))
    
    ;; Update balances and supply
    (call $set_balance 
      (local.get $caller) 
      (i32.sub (local.get $caller_balance) (local.get $amount)))
    
    (call $set_total_supply (i32.sub (local.get $current_supply) (local.get $amount)))
    
    ;; Log success
    (call $log (i32.const 136) (i32.const 15))
    
    (i32.const 1)
  )

  ;; Helper functions for storage operations

  ;; Get total supply from storage
  (func $get_total_supply (result i32)
    (local $length i32)
    
    (local.set $length 
      (call $storage_get 
        (global.get $total_supply_key_ptr)
        (global.get $total_supply_key_len)))
    
    (if (i32.gt_u (local.get $length) (i32.const 0))
      (then (i32.load (i32.const 200)))
      (else (i32.const 0))
    )
  )

  ;; Set total supply in storage
  (func $set_total_supply (param $supply i32)
    (i32.store (i32.const 200) (local.get $supply))
    (call $storage_set 
      (global.get $total_supply_key_ptr)
      (global.get $total_supply_key_len)
      (i32.const 200)
      (i32.const 4))
  )

  ;; Get balance for an address
  (func $get_balance (param $address i32) (result i32)
    (local $length i32)
    
    ;; Create storage key: "balance_" + address
    ;; For simplicity, we'll use the address as-is
    ;; In practice, you'd create a proper key
    
    (local.set $length 
      (call $storage_get 
        (global.get $balance_prefix_ptr)
        (i32.add (global.get $balance_prefix_len) (i32.const 4))))  ;; simplified
    
    (if (i32.gt_u (local.get $length) (i32.const 0))
      (then (i32.load (i32.const 300)))
      (else (i32.const 0))
    )
  )

  ;; Set balance for an address
  (func $set_balance (param $address i32) (param $balance i32)
    (i32.store (i32.const 300) (local.get $balance))
    (call $storage_set 
      (global.get $balance_prefix_ptr)
      (i32.add (global.get $balance_prefix_len) (i32.const 4))  ;; simplified
      (i32.const 300)
      (i32.const 4))
  )
)
