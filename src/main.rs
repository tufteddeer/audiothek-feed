use audiothek::program_set;
use axum::{extract::Path, routing::get, Router};

mod audiothek;

async fn get_atom_feed(Path(id): Path<String>) -> axum::response::Response<String> {
    let feed = audiothek::fetch_feed(program_set::Variables { id })
        .await
        .unwrap();

    axum::response::Response::new(feed.to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new().route("/feed/:id", get(get_atom_feed));

    let socket = "0.0.0.0:3000";

    println!("Listening on http://{socket}");
    axum::Server::bind(&socket.parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
