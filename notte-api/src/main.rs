use std::path::PathBuf;

use axum::{Json, Router, http::StatusCode, routing::post};
use serde::{Deserialize, Serialize};
use tokio::io;

type ApiReturn<T> = (StatusCode, Json<T>);

#[derive(Deserialize)]
struct OpenRequest {
    path: String,
}

#[derive(Serialize)]
enum FileType {
    File(FileEntry),
    Dir(DirEntry),
    Link(LinkEntry),
    Other,
}

#[derive(Serialize)]
struct FileEntry {
    name: String,
    path: String,

    content: Vec<String>,
}

#[derive(Serialize)]
struct DirEntry {
    name: String,
    path: String,

    content: Vec<FileType>,
}

#[derive(Serialize)]
struct LinkEntry {
    name: String,
    path: String,

    link: FileEntry,
}

#[derive(Serialize)]
struct OpenResponse {
    entry: Option<FileType>,
}

async fn create_file_entry(path: &PathBuf) -> io::Result<FileEntry> {
    let abs_path = path.canonicalize().unwrap_or(path.clone());
    let content = tokio::fs::read_to_string(path).await?;

    Ok(FileEntry {
        name: path.display().to_string(),
        path: abs_path.display().to_string(),

        content: content.lines().map(|l| l.to_string()).collect(),
    })
}

async fn create_file_entry_with_status(path: &PathBuf) -> (StatusCode, Option<FileEntry>) {
    let (status, entry) = match create_file_entry(path).await {
        Ok(e) => (StatusCode::OK, Some(e)),
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => (StatusCode::NOT_FOUND, None),
            io::ErrorKind::PermissionDenied => (StatusCode::FORBIDDEN, None),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, None),
        },
    };
    (status, entry)
}

async fn create_dir_entry(path: &PathBuf) -> io::Result<DirEntry> {
    let abs_path = path.canonicalize().unwrap_or(path.clone());
    let mut dir = tokio::fs::read_dir(path).await?;

    let mut content: Vec<FileType> = Vec::new();

    while let Ok(Some(entry)) = dir.next_entry().await {
        let md = entry.metadata().await?;
        let file_type = if md.is_dir() {
            FileType::Dir(create_dir_entry(&entry.path()).await?)
        } else if md.is_file() {
            FileType::File(create_file_entry(&entry.path()).await?)
        } else {
            FileType::Other
        };
        content.push(file_type)
    }

    Ok(DirEntry {
        name: path.display().to_string(),
        path: abs_path.display().to_string(),

        content: content,
    })
}

async fn create_dir_entry_with_status(path: &PathBuf) -> (StatusCode, Option<DirEntry>) {
    let (status, entry) = match create_dir_entry(path).await {
        Ok(e) => (StatusCode::OK, Some(e)),
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => (StatusCode::NOT_FOUND, None),
            io::ErrorKind::PermissionDenied => (StatusCode::FORBIDDEN, None),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, None),
        },
    };
    (status, entry)
}

async fn create_link_entry(path: &PathBuf) -> io::Result<LinkEntry> {
    unimplemented!()
}

async fn create_link_entry_with_status(path: &PathBuf) -> (StatusCode, Option<LinkEntry>) {
    unimplemented!()
}

async fn create_file_type(path: &PathBuf) -> (StatusCode, Option<FileType>) {
    let (status, file_type) = match tokio::fs::metadata(path).await {
        Ok(md) => {
            if md.is_file() {
                let (status, file) = create_file_entry_with_status(path).await;
                if let Some(file) = file {
                    (status, Some(FileType::File(file)))
                } else {
                    (status, None)
                }
            } else if md.is_dir() {
                let (status, dir) = create_dir_entry_with_status(path).await;
                if let Some(dir) = dir {
                    (status, Some(FileType::Dir(dir)))
                } else {
                    (status, None)
                }
            } else if md.is_symlink() {
                let (status, link) = create_link_entry_with_status(path).await;
                if let Some(link) = link {
                    (status, Some(FileType::Link(link)))
                } else {
                    (status, None)
                }
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, None)
            }
        }
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => (StatusCode::NOT_FOUND, None),
            io::ErrorKind::PermissionDenied => (StatusCode::FORBIDDEN, None),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, None),
        },
    };

    (status, file_type)
}

async fn api_open(Json(open_request): Json<OpenRequest>) -> ApiReturn<OpenResponse> {
    let path = PathBuf::from(&open_request.path);

    let (status, file_type) = create_file_type(&path).await;

    (status, Json(OpenResponse { entry: file_type }))
}

#[derive(Deserialize)]
struct LsRequest {
    path: String,
}

#[derive(Serialize)]
struct LsResponse {
    content: Option<DirEntry>,
}

async fn api_ls(Json(ls_request): Json<LsRequest>) -> ApiReturn<LsResponse> {
    let path = PathBuf::from(&ls_request.path);
    let (status, dir) = create_dir_entry_with_status(&path).await;

    (status, Json(LsResponse { content: dir }))
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
