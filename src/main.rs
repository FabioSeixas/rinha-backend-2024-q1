use axum::{routing::get, Router};
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    Row,
};

#[derive(Debug)]
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

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://admin:123@localhost/rinha")
        .await?;

    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL/MariaDB)

    let app = Router::new().route(
        "/",
        get(|| async move {
            let row = sqlx::query("SELECT * from clientes")
                .map(|row: PgRow| Cliente::new(row.get(0), row.get(1), row.get(2)))
                .fetch_all(&pool)
                .await
                .expect("Error getting clients");

            println!("{:?}", row);

            "hello world"
        }),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
