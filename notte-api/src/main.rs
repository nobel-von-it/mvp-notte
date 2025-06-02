use axum::{Json, Router, http::StatusCode, routing::post};
use serde::{Deserialize, Serialize};
use tokio::io;

type ApiReturn<T> = (StatusCode, Json<T>);

#[derive(Deserialize)]
struct OpenRequest {
    path: String,
}

#[derive(Serialize)]
struct OpenResponse {
    content: Option<Vec<String>>,
}

async fn api_open(Json(open_request): Json<OpenRequest>) -> ApiReturn<OpenResponse> {
    let result = tokio::fs::read_to_string(&open_request.path).await;

    let status = match &result {
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
            io::ErrorKind::PermissionDenied => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        },
        Ok(_) => StatusCode::OK,
    };

    let content = result
        .map(|c| Some(c.lines().map(|l| l.to_string()).collect::<Vec<String>>()))
        .unwrap_or(None);

    (status, Json(OpenResponse { content }))
}

#[derive(Deserialize)]
struct LsRequest {
    path: String,
}

#[derive(Serialize)]
struct LsResponse {
    path: String,
    content: Option<Vec<String>>,
}

async fn api_ls(Json(ls_request): Json<LsRequest>) -> ApiReturn<LsResponse> {
    let result = tokio::fs::read_dir(&ls_request.path).await;

    let (content, status) = match result {
        Ok(mut dir) => {
            let mut files = Vec::new();
            while let Ok(Some(entry)) = dir.next_entry().await {
                files.push(entry.file_name().to_string_lossy().to_string());
            }
            (Some(files), StatusCode::OK)
        }
        Err(e) => {
            let status = match e.kind() {
                io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
                io::ErrorKind::PermissionDenied => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (None, status)
        }
    };

    (
        status,
        Json(LsResponse {
            path: ls_request.path,
            content,
        }),
    )
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let router = Router::new()
        .route("/api/open", post(api_open))
        .route("/api/ls", post(api_ls))
        .layer(tower_http::cors::CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;

    axum::serve(listener, router).await?;
    Ok(())
}
