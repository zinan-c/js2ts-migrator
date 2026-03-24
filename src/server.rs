use std::net::SocketAddr;

use anyhow::{Context, Error};
use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::Serialize;
use tower_http::services::ServeDir;
use tokio::net::TcpListener;

use crate::migrator;

#[derive(Serialize)]
struct MigrateResponse {
    content: String,
    var_count: usize,
    line_count: usize,
    output_name: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn run(port: u16) -> Result<(), Error> {
    let app = Router::new()
        .route("/api/migrate", post(migrate_handler))
        .nest_service("/", ServeDir::new("web"));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("web ui: http://{addr}");

    let listener = TcpListener::bind(addr)
        .await
        .context("bind server port")?;
    axum::serve(listener, app)
        .await
        .context("server failed")?;
    Ok(())
}

async fn migrate_handler(mut multipart: Multipart) -> Response {
    match migrate_from_multipart(&mut multipart).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: err.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn migrate_from_multipart(multipart: &mut Multipart) -> Result<MigrateResponse, Error> {
    let mut file_name = None;
    let mut file_bytes = None;

    while let Some(field) = multipart.next_field().await? {
        if field.name() != Some("file") {
            continue;
        }
        file_name = field.file_name().map(|s| s.to_string());
        file_bytes = Some(field.bytes().await?);
        break;
    }

    let file_name = file_name.ok_or_else(|| Error::msg("missing file name"))?;
    let bytes = file_bytes.ok_or_else(|| Error::msg("missing file data"))?;
    let source = String::from_utf8(bytes.to_vec()).context("file must be valid UTF-8")?;

    let module = migrator::parse_js_source(&source, &file_name)?;
    let type_map = migrator::infer_var_types(&module);
    let var_count = migrator::count_var_decls(&module);
    let content = migrator::generate_ts(module, &type_map);
    let line_count = count_lines(&source);

    let output_name = file_name
        .rsplit_once('.')
        .map(|(base, _)| format!("{base}.ts"))
        .unwrap_or_else(|| format!("{file_name}.ts"));

    Ok(MigrateResponse {
        content,
        var_count,
        line_count,
        output_name,
    })
}

fn count_lines(source: &str) -> usize {
    if source.is_empty() {
        0
    } else {
        source.lines().count()
    }
}
