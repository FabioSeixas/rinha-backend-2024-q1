use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{
    postgres::{ PgPoolOptions, PgRow},
    Postgres, Row,
};

use types::{CreateTransactionPayload, Transaction, TransactionType};

mod types;

#[derive(Debug, Serialize, Deserialize)]
struct Cliente {
    id: i32,
    nome: String,
    limite: i32,
}

impl Cliente {
    fn new(id: i32, nome: String, limite: i32) -> Self {
        Self { id, nome, limite }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Saldo {
    id: i32,
    cliente_id: i32,
    valor: i32,
}

impl Saldo {
    fn new(id: i32, cliente_id: i32, valor: i32) -> Self {
        Self {
            id,
            cliente_id,
            valor,
        }
    }
}

async fn client_exist(
    db: &sqlx::pool::Pool<Postgres>,
    client_id: &str,
) -> Result<Cliente, sqlx::Error> {
    sqlx::query("SELECT * from clientes WHERE id = $1")
        .bind(client_id.parse::<i32>().unwrap())
        .map(|row: PgRow| Cliente::new(row.get(0), row.get(1), row.get(2)))
        .fetch_one(db)
        .await
}

async fn get_current_saldo(
    trx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    client_id: &str,
) -> Result<i32, sqlx::Error> {
    sqlx::query(
        "SELECT valor FROM saldos 
         WHERE cliente_id = $1",
    )
    .bind(client_id.parse::<i32>().unwrap())
    .map(|row: PgRow| row.get(0))
    .fetch_one(&mut **trx)
    .await
}

#[debug_handler]
async fn get_clientes(State(state): State<sqlx::pool::Pool<Postgres>>) -> impl IntoResponse {
    let clientes = sqlx::query("SELECT * from clientes")
        .map(|row: PgRow| Cliente::new(row.get(0), row.get(1), row.get(2)))
        .fetch_all(&state)
        .await
        .expect("Error getting clients");

    Json(serde_json::to_value(&clientes).unwrap())
}

#[debug_handler]
async fn extrato(
    State(state): State<sqlx::pool::Pool<Postgres>>,
    Path(cliente_id): Path<String>,
) -> impl IntoResponse {
    match client_exist(&state, &cliente_id).await {
        Ok(cliente) => {
            let mut trx = state.begin().await.unwrap();
            let saldo_atual = get_current_saldo(&mut trx, &cliente_id)
                .await
                .expect("Error while getting current saldo");

            let last_transactions: Vec<Transaction> = sqlx::query(
                "SELECT valor, tipo, descricao, realizada_em::TEXT
                 FROM transacoes
                 WHERE cliente_id = $1
                 LIMIT 10",
            )
            .bind(cliente_id.parse::<i32>().unwrap())
            .map(|row: PgRow| Transaction::new(row.get(0), row.get(1), row.get(2), row.get(3)))
            .fetch_all(&mut *trx)
            .await
            .expect("error while getting transactions");

            trx.commit().await.unwrap();

            let date = std::time::SystemTime::now();
            (
                StatusCode::OK,
                Json(json!({
                    "saldo": {
                        "total": saldo_atual,
                        "data_extrato": date,
                        "limite": cliente.limite
                    },
                    "ultimas_transacoes": last_transactions

                })),
            )
        }
        Err(_) => {
            println!("ERROR!");
            (
                StatusCode::from_u16(404).unwrap(),
                Json(json!({
                    "message":"cliente inexistente"
                })),
            )
        }
    }
}

#[debug_handler]
async fn transaction(
    State(state): State<sqlx::pool::Pool<Postgres>>,
    Path(client_id): Path<String>,
    Json(payload): Json<CreateTransactionPayload>,
) -> impl IntoResponse {
    println!("{:?}", payload);
    println!("{:?}", client_id);
    match client_exist(&state, &client_id).await {
        Ok(cliente) => {
            println!("OK!");

            let mut trx = state.begin().await.unwrap();

            let saldo_atual = get_current_saldo(&mut trx, &client_id)
                .await
                .expect("Error while getting current saldo");

            println!("saldo atual: {saldo_atual}");

            let transaction_type = if payload.tipo == "d" {
                TransactionType::Debit
            } else {
                TransactionType::Credit
            };

            let new_saldo = match transaction_type {
                types::TransactionType::Debit => {
                    let new_saldo = saldo_atual - payload.valor;
                    println!("new saldo: {new_saldo}");
                    if new_saldo < 0 {
                        if new_saldo.abs() > cliente.limite {
                            return (
                                StatusCode::from_u16(422).unwrap(),
                                Json(json!({
                                    "message":"sem limite disponivel"
                                })),
                            );
                        }
                    }

                    sqlx::query(
                        "UPDATE saldos 
                         SET valor = valor - $1
                         WHERE cliente_id = $2",
                    )
                    .bind(payload.valor)
                    .bind(client_id.parse::<i32>().unwrap())
                    .execute(&mut *trx)
                    .await
                    .expect("error while updating saldo");
                    new_saldo
                }
                types::TransactionType::Credit => {
                    let new_saldo = sqlx::query(
                        "UPDATE saldos 
                         SET valor = valor + $1
                         WHERE cliente_id = $2
                         RETURNING valor",
                    )
                    .bind(payload.valor)
                    .bind(client_id.parse::<i32>().unwrap())
                    .map(|row: PgRow| row.get(0))
                    .fetch_one(&mut *trx)
                    .await
                    .expect("error while updating saldo");
                    new_saldo
                }
            };

            sqlx::query(
                "INSERT INTO transacoes 
                 (cliente_id, valor, tipo, descricao)
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(client_id.parse::<i32>().unwrap())
            .bind(&payload.valor)
            .bind(&payload.tipo)
            .bind(&payload.descricao)
            .execute(&mut *trx)
            .await
            .expect("Error while inserting transacoes");

            trx.commit()
                .await
                .expect("Error while commiting transaction");

            (
                StatusCode::OK,
                Json(json!({
                    "limite": cliente.limite,
                    "saldo": new_saldo
                })),
            )
        }
        Err(_) => {
            println!("ERROR!");
            (
                StatusCode::from_u16(404).unwrap(),
                Json(json!({
                    "message":"cliente inexistente"
                })),
            )
        }
    }

    // Json(serde_json::to_value(&saldos).unwrap())
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect("postgres://admin:123@localhost/rinha")
        .await?;

    let app = Router::new()
        .route("/clientes", get(get_clientes))
        .route("/clientes/:id/extrato", get(extrato))
        .route("/clientes/:id/transacoes", post(transaction))
        .with_state(pool.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
