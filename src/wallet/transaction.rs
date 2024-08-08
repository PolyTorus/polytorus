use uuid::Uuid;

pub struct Transaction {
    pub id: Uuid,
    pub input: Vec<Input>,
    pub output: Vec<Output>,
}

pub struct Input {
    pub transaction_id: Uuid,
    pub index: u32,
    pub amount: u64,
    pub address: String,
}

pub struct Output {
    pub index: u32,
    pub amount: u64,
    pub address: String,
}

impl Transaction {
    pub fn new(input: Vec<Input>, output: Vec<Output>) -> Self {
        Transaction {
            id: Uuid::new_v4(),
            input,
            output,
        }
    }

    pub fn add_input(&mut self, transaction_id: Uuid, index: u32, amount: u64, address: String) {
        self.input.push(Input {
            transaction_id,
            index,
            amount,
            address,
        });
    }

    pub fn add_output(&mut self, index: u32, amount: u64, address: String) {
        self.output.push(Output { index, amount, address });
    }

    pub fn get_balance(&self, address: &str) -> i64 {
        let mut balance = 0;
        for input in &self.input {
            if input.address == address {
                balance -= input.amount as i64;
            }
        }
        for output in &self.output {
            if output.address == address {
                balance += output.amount as i64;
            }
        }
        balance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction() {
        let mut transaction = Transaction::new(vec![], vec![]);
        transaction.add_input(Uuid::new_v4(), 0, 100, "Alice".to_string());
        transaction.add_output(0, 100, "Bob".to_string());
        assert_eq!(transaction.get_balance("Alice"), -100);
        assert_eq!(transaction.get_balance("Bob"), 100);
    }
}