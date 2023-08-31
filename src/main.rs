use audiothek::program_set;
use audiothek_feed::add_template_file;
use axum::{
    extract::{Host, Path, Query},
    response::IntoResponse,
    routing::get,
    Router,
};
use lazy_static::lazy_static;

mod audiothek;

use serde::Deserialize;
use tera::{Context, Tera};
use tracing_subscriber::fmt::format::FmtSpan;
use url::Url;

#[derive(Deserialize, Debug)]
struct FeedQuery {
    show: String,
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
async fn feed_info_view(Host(hostname): Host, query: Query<FeedQuery>) -> impl IntoResponse {
    fn id_from_url(url: &str) -> anyhow::Result<String> {
        let u = Url::parse(url)?;
        let mut seg = u
            .path_segments()
            .ok_or(anyhow::anyhow!("No path segments"))?;
        let id = seg.nth(2).ok_or(anyhow::anyhow!("Not enough segments"))?;

        Ok(id.to_string())
    }

    let show_param = query.0.show;

    let id = if let Ok(id) = id_from_url(&show_param) {
        id
    } else {
        show_param
    };

    let response =
        audiothek::fetch_metadata(audiothek::program_metadata::Variables { id: id.clone() }).await;

    let mut context = Context::new();

    let template_file = match response {
        Ok(meta) => {
            context.insert("url", &format!("{hostname}/feed/{}", id));
            context.insert("title", &meta.title);

            if let Some(url) = meta.image.and_then(|img| img.url) {
                context.insert("image", &audiothek::image_url(&url, 512));
            }

            "feed_info_view.html"
        }
        Err(e) => {
            context.insert("message", &e.to_string());
            "feed_info_error.html"
        }
    };

    let res = TEMPLATES.render(template_file, &context).unwrap();

    axum::response::Response::new(res)
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();

        add_template_file!(tera, env!("FRONTEND_DIR"), "feed_info_view.html")
            .expect("Failedtera,  to add template");
        add_template_file!(tera, env!("FRONTEND_DIR"), "feed_info_error.html")
            .expect("Failed to add template");

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
    tracing_subscriber::fmt::fmt()
        .with_env_filter("hyper=warn,audiothek_feed=trace")
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .init();

    let app = Router::new()
        .route("/feed/:id", get(get_atom_feed))
        .route("/", get(index_handler))
        .route("/feed-info", get(feed_info_view))
        .route("/style.css", get(css_file));

    let socket = "0.0.0.0:3000";

    tracing::info!("Listening on http://{socket}");
    axum::Server::bind(&socket.parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
