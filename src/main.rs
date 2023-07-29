use audiothek::program_set;
use axum::{
    extract::{Host, Path, Query},
    routing::get,
    Router,
};
use lazy_static::lazy_static;
use simple_logger::SimpleLogger;

mod audiothek;

use serde::Deserialize;
use tera::{Context, Tera};

#[derive(Deserialize, Debug)]
struct FeedQuery {
    id: String,
}

/// Serves the XML Atom feed
async fn get_atom_feed(Path(id): Path<String>) -> axum::response::Response<String> {
    let feed = audiothek::fetch_feed(program_set::Variables { id })
        .await
        .unwrap();

    axum::response::Response::new(feed.to_string())
}

async fn index_handler() -> axum::response::Response<String> {
    axum::response::Response::new(
        include_str!(concat!(env!("FRONTEND_DIR"), "/index.html")).to_string(),
    )
}

/// Serves the HTML UI with show metadata and url
async fn feed_info_view(
    Host(hostname): Host,
    id: Query<FeedQuery>,
) -> axum::response::Response<String> {
    let meta = audiothek::fetch_metadata(audiothek::program_metadata::Variables {
        id: id.0.id.clone(),
    })
    .await
    .unwrap();

    let mut context = Context::new();
    context.insert("url", &format!("{hostname}/feed/{}", id.0.id));
    context.insert("title", &meta.title);

    if let Some(url) = meta.image.and_then(|img| img.url) {
        context.insert("image", &audiothek::image_url(&url, 512));
    }

    let res = TEMPLATES.render("feed_url.html", &context).unwrap();

    axum::response::Response::new(res)
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();

        if let Err(e) = tera.add_raw_template(
            "feed_url.html",
            include_str!(concat!(env!("FRONTEND_DIR"), "/feed_info_view.html")),
        ) {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }

        tera.autoescape_on(vec![".html"]);
        tera
    };
}

async fn css_file() -> axum::response::Response<String> {
    axum::response::Response::new(
        include_str!(concat!(env!("FRONTEND_DIR"), "/style.css")).to_string(),
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", env! {"FRONTEND_DIR"});

    SimpleLogger::new().init()?;

    let app = Router::new()
        .route("/feed/:id", get(get_atom_feed))
        .route("/", get(index_handler))
        .route("/feed-info", get(feed_info_view))
        .route("/style.css", get(css_file));

    let socket = "0.0.0.0:3123";

    println!("Listening on http://{socket}");
    axum::Server::bind(&socket.parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
