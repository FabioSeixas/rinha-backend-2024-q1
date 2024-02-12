use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionType {
    Credit,
    Debit
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransactionPayload {
    pub valor: i32,
    pub tipo: String,
    pub descricao: String,
}
