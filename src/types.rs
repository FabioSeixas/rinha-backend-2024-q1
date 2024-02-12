use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionType {
    Credit,
    Debit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransactionPayload {
    pub valor: i32,
    pub tipo: String,
    pub descricao: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub valor: i32,
    pub tipo: String,
    pub descricao: String,
    pub realizada_em: String,
}

impl Transaction {
    pub fn new(valor: i32, tipo: String, descricao: String, realizada_em: String) -> Self {
        Self {
            realizada_em,
            tipo,
            valor,
            descricao,
        }
    }
}
