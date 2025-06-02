;; Counter contract - demonstrates state management and function calls
(module
  ;; Import host functions
  (import "env" "storage_get" (func $storage_get (param i32 i32) (result i32)))
  (import "env" "storage_set" (func $storage_set (param i32 i32 i32 i32)))
  (import "env" "log" (func $log (param i32 i32)))
  (import "env" "get_caller" (func $get_caller (result i32)))
  (import "env" "get_value" (func $get_value (result i64)))

  ;; Memory for contract operations
  (memory 1)
  
  ;; Global variables
  (global $counter_key_ptr i32 (i32.const 0))
  (global $counter_key_len i32 (i32.const 7))
  
  ;; String constants
  (data (i32.const 0) "counter")
  (data (i32.const 8) "Counter incremented to: ")
  (data (i32.const 32) "Counter initialized")
  (data (i32.const 50) "Current counter value: ")

  ;; Initialize the counter contract
  (func (export "init") (result i32)
    ;; Set initial counter value to 0
    (call $storage_set 
      (global.get $counter_key_ptr)  ;; key pointer
      (global.get $counter_key_len)  ;; key length
      (i32.const 100)                ;; value pointer (store 0 at memory[100])
      (i32.const 4))                 ;; value length (4 bytes for i32)
    
    ;; Store initial value 0 at memory[100]
    (i32.store (i32.const 100) (i32.const 0))
    
    ;; Log initialization
    (call $log (i32.const 32) (i32.const 17))
    
    (i32.const 1) ;; return success
  )

  ;; Increment the counter
  (func (export "increment") (result i32)
    (local $current_value i32)
    
    ;; Get current counter value
    (local.set $current_value (call $get_counter_value))
    
    ;; Increment the value
    (local.set $current_value (i32.add (local.get $current_value) (i32.const 1)))
    
    ;; Store the new value
    (call $set_counter_value (local.get $current_value))
    
    ;; Log the increment
    (call $log_counter_value (local.get $current_value))
    
    (local.get $current_value) ;; return new value
  )

  ;; Get the current counter value
  (func (export "get") (result i32)
    (call $get_counter_value)
  )

  ;; Add a specific value to the counter
  (func (export "add") (param $amount i32) (result i32)
    (local $current_value i32)
    
    ;; Get current value
    (local.set $current_value (call $get_counter_value))
    
    ;; Add the amount
    (local.set $current_value (i32.add (local.get $current_value) (local.get $amount)))
    
    ;; Store the new value
    (call $set_counter_value (local.get $current_value))
    
    ;; Log the new value
    (call $log_counter_value (local.get $current_value))
    
    (local.get $current_value)
  )

  ;; Reset counter to zero
  (func (export "reset") (result i32)
    ;; Set counter to 0
    (call $set_counter_value (i32.const 0))
    
    ;; Log reset
    (call $log (i32.const 32) (i32.const 17))
    
    (i32.const 0)
  )

  ;; Helper function to get counter value from storage
  (func $get_counter_value (result i32)
    (local $length i32)
    
    ;; Try to get the value from storage
    (local.set $length 
      (call $storage_get 
        (global.get $counter_key_ptr)
        (global.get $counter_key_len)))
    
    ;; If we got data, load it from memory
    (if (i32.gt_u (local.get $length) (i32.const 0))
      (then
        ;; For this simple implementation, assume the value is stored at a known location
        ;; In a real implementation, storage_get would write to a specified memory location
        (i32.load (i32.const 100))
      )
      (else
        ;; No data found, return 0
        (i32.const 0)
      )
    )
  )

  ;; Helper function to set counter value in storage
  (func $set_counter_value (param $value i32)
    ;; Store the value in memory first
    (i32.store (i32.const 100) (local.get $value))
    
    ;; Then save to persistent storage
    (call $storage_set 
      (global.get $counter_key_ptr)  ;; key pointer
      (global.get $counter_key_len)  ;; key length
      (i32.const 100)               ;; value pointer
      (i32.const 4))                ;; value length
  )

  ;; Helper function to log counter value
  (func $log_counter_value (param $value i32)
    ;; Store the message prefix
    (call $log (i32.const 50) (i32.const 22))
    
    ;; In a real implementation, we'd format the number and log it
    ;; For now, just indicate the operation happened
  )
)
