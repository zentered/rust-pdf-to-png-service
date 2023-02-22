pub mod convert;
pub mod storage;
pub mod transform;

use crate::convert::convert;
use crate::storage::{download, upload};
use crate::transform::{init, load_image, transform};

use axum::{
    debug_handler,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};

use dotenv;
use futures::future::try_join_all;
use libvips::VipsApp;
use opentelemetry::sdk::export::trace::stdout;
use serde::Deserialize;
use std::io::Cursor;
use std::net::SocketAddr;
use std::{env, sync::Arc};
use tower_http::trace::TraceLayer;
use tracing::{debug, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

pub enum Format {
    Original,
    Preview,
    Thumbnail,
    WebP,
    WebPLossless,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    dotenv::dotenv().ok();
    let tracer = stdout::new_pipeline().install_simple();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("DEBUG"));
    let subscriber = Registry::default();
    subscriber.with(telemetry).with(env_filter).init();

    let addr = SocketAddr::from((
        [0, 0, 0, 0],
        env::var("PORT").unwrap().parse::<u16>().unwrap(),
    ));
    let app_state = AppState {
        vips: Arc::new(init()),
    };

    let app = Router::new()
        .route("/", post(service))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);
    debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}

#[derive(Deserialize)]
struct ArcTrigger {
    name: String,
    // bucket: String,
}

#[derive(Clone)]
pub struct AppState {
    vips: Arc<VipsApp>,
}

#[debug_handler]
async fn service(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ArcTrigger>,
) -> Response {
    debug!("service started... ");
    if headers.get("ce-subject").is_none() {
        return (StatusCode::BAD_REQUEST, ()).into_response();
    }
    info!("Headers: {:?}", headers);
    info!("Payload: {:?}", payload.name);
    let filename = payload.name.clone();
    let file = download(&filename).await.unwrap();

    let files = process(state.vips, &filename, file);

    let upload_futures = files.iter().map(|(name, content)| {
        // magic with futures :D
        upload(&name, content.clone())
    });

    match try_join_all(upload_futures).await {
        Ok(_) => (StatusCode::OK, "All good").into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to upload file: {err}"),
        )
            .into_response(),
    }
}

fn process(vips_app: Arc<VipsApp>, filename: &str, pdf_buffer: Vec<u8>) -> Vec<(String, Vec<u8>)> {
    let name = filename.clone().replace(".pdf", "");
    let original_png = convert(pdf_buffer);
    info!("Converted PDF");
    let mut bytes: Vec<u8> = Vec::new();
    original_png
        .write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)
        .unwrap();

    let original_image = load_image(&bytes);
    let full_size = transform(&original_image, Format::Original).unwrap_or_else(|_| {
        println!("{}", vips_app.error_buffer().unwrap());
        Vec::new()
    });
    let preview = transform(&original_image, Format::Preview).unwrap_or_else(|_| {
        println!("{}", vips_app.error_buffer().unwrap());
        Vec::new()
    });
    let thumbnail = transform(&original_image, Format::Thumbnail).unwrap_or_else(|_| {
        println!("{}", vips_app.error_buffer().unwrap());
        Vec::new()
    });
    let webp = transform(&original_image, Format::WebP).unwrap_or_else(|_| {
        println!("{}", vips_app.error_buffer().unwrap());
        Vec::new()
    });
    let webplossless = transform(&original_image, Format::WebPLossless).unwrap_or_else(|_| {
        println!("{}", vips_app.error_buffer().unwrap());
        Vec::new()
    });
    vec![
        (format!("{}-original.png", name), bytes),
        (format!("{}.png", name), full_size),
        (format!("{}-preview.png", name), preview),
        (format!("{}-small.png", name), thumbnail),
        (format!("{}.webp", name), webp),
        (format!("{}-lossless.webp", name), webplossless),
    ]
}
