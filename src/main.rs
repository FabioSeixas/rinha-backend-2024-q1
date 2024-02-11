use axum::{
    http::StatusCode,
    debug_handler,
    extract::{State},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    Postgres, Row,
};

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
        Self { id, cliente_id, valor }
    }
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
async fn get_saldos(State(state): State<sqlx::pool::Pool<Postgres>>) -> impl IntoResponse {
    let saldos = sqlx::query("SELECT * from saldos")
        .map(|row: PgRow| Saldo::new(row.get(0), row.get(1), row.get(2)))
        .fetch_all(&state)
        .await
        .expect("Error getting clients");

    Json(serde_json::to_value(&saldos).unwrap())
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect("postgres://admin:123@localhost/rinha")
        .await?;

    let app = Router::new()
        .route("/clientes", get(get_clientes))
        .route("/saldos", get(get_saldos))
        .with_state(pool.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
