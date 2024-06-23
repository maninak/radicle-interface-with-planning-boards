use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use hyper::HeaderMap;
use radicle_surf::blob::{Blob, BlobRef};
use tower_http::cors;

use radicle::prelude::RepoId;
use radicle::profile::Profile;
use radicle::storage::{ReadRepository, ReadStorage};
use radicle_surf::{Oid, Repository};

use crate::api::RawQuery;
use crate::axum_extra::Path;
use crate::error::RawError as Error;

const MAX_BLOB_SIZE: usize = 4_194_304;

static MIMES: &[(&str, &str)] = &[
    ("3gp", "video/3gpp"),
    ("7z", "application/x-7z-compressed"),
    ("aac", "audio/aac"),
    ("avi", "video/x-msvideo"),
    ("bin", "application/octet-stream"),
    ("bmp", "image/bmp"),
    ("bz", "application/x-bzip"),
    ("bz2", "application/x-bzip2"),
    ("csh", "application/x-csh"),
    ("css", "text/css"),
    ("csv", "text/csv"),
    ("doc", "application/msword"),
    (
        "docx",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ),
    ("epub", "application/epub+zip"),
    ("gz", "application/gzip"),
    ("gif", "image/gif"),
    ("htm", "text/html"),
    ("html", "text/html"),
    ("ico", "image/vnd.microsoft.icon"),
    ("jar", "application/java-archive"),
    ("jpeg", "image/jpeg"),
    ("jpg", "image/jpeg"),
    ("js", "text/javascript"),
    ("json", "application/json"),
    ("mjs", "text/javascript"),
    ("mp3", "audio/mpeg"),
    ("mp4", "video/mp4"),
    ("mpeg", "video/mpeg"),
    ("odp", "application/vnd.oasis.opendocument.presentation"),
    ("ods", "application/vnd.oasis.opendocument.spreadsheet"),
    ("odt", "application/vnd.oasis.opendocument.text"),
    ("oga", "audio/ogg"),
    ("ogv", "video/ogg"),
    ("ogx", "application/ogg"),
    ("otf", "font/otf"),
    ("png", "image/png"),
    ("pdf", "application/pdf"),
    ("php", "application/x-httpd-php"),
    ("ppt", "application/vnd.ms-powerpoint"),
    (
        "pptx",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    ),
    ("rar", "application/vnd.rar"),
    ("rtf", "application/rtf"),
    ("sh", "application/x-sh"),
    ("svg", "image/svg+xml"),
    ("tar", "application/x-tar"),
    ("tif", "image/tiff"),
    ("tiff", "image/tiff"),
    ("ttf", "font/ttf"),
    ("txt", "text/plain"),
    ("wav", "audio/wav"),
    ("weba", "audio/webm"),
    ("webm", "video/webm"),
    ("webp", "image/webp"),
    ("woff", "font/woff"),
    ("woff2", "font/woff2"),
    ("xhtml", "application/xhtml+xml"),
    ("xls", "application/vnd.ms-excel"),
    (
        "xlsx",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    ),
    ("xml", "application/xml"),
    ("zip", "application/zip"),
];

pub fn router(profile: Arc<Profile>) -> Router {
    Router::new()
        .route("/:rid/:sha/*path", get(file_by_commit_handler))
        .route("/:rid/head/*path", get(file_by_canonical_head_handler))
        .route("/:rid/blobs/:oid", get(file_by_oid_handler))
        .with_state(profile)
        .layer(
            cors::CorsLayer::new()
                .max_age(Duration::from_secs(86400))
                .allow_origin(cors::Any)
                .allow_methods([Method::GET])
                .allow_headers([header::CONTENT_TYPE]),
        )
}

async fn file_by_commit_handler(
    Path((rid, sha, path)): Path<(RepoId, Oid, String)>,
    State(profile): State<Arc<Profile>>,
) -> impl IntoResponse {
    let storage = &profile.storage;
    let repo = storage.repository(rid)?;

    // Don't allow downloading raw files for private repos.
    if repo.identity_doc()?.visibility.is_private() {
        return Err(Error::NotFound);
    }

    let repo: Repository = repo.backend.into();
    let blob = repo.blob(sha, &path)?;

    blob_response(blob, path)
}

async fn file_by_canonical_head_handler(
    Path((rid, path)): Path<(RepoId, String)>,
    State(profile): State<Arc<Profile>>,
) -> impl IntoResponse {
    let storage = &profile.storage;
    let repo = storage.repository(rid)?;

    // Don't allow downloading raw files for private repos.
    if repo.identity_doc()?.visibility.is_private() {
        return Err(Error::NotFound);
    }

    let (_, sha) = repo.head()?;
    let repo: Repository = repo.backend.into();
    let blob = repo.blob(sha, &path)?;

    blob_response(blob, path)
}

fn blob_response(
    blob: Blob<BlobRef>,
    path: String,
) -> Result<(StatusCode, HeaderMap, Vec<u8>), Error> {
    let mut response_headers = HeaderMap::new();
    if blob.size() > MAX_BLOB_SIZE {
        return Ok::<_, Error>((StatusCode::PAYLOAD_TOO_LARGE, response_headers, vec![]));
    }

    let mime = if let Some(ext) = path.split('.').last() {
        MIMES
            .binary_search_by(|(k, _)| k.cmp(&ext))
            .map(|k| MIMES[k].1)
            .unwrap_or("text; charset=utf-8")
    } else {
        "application/octet-stream"
    };
    response_headers.insert(header::CONTENT_TYPE, HeaderValue::from_str(mime)?);

    Ok::<_, Error>((StatusCode::OK, response_headers, blob.content().to_owned()))
}

async fn file_by_oid_handler(
    Path((rid, oid)): Path<(RepoId, Oid)>,
    State(profile): State<Arc<Profile>>,
    Query(qs): Query<RawQuery>,
) -> impl IntoResponse {
    let storage = &profile.storage;
    let repo = storage.repository(rid)?;

    // Don't allow downloading raw files for private repos.
    if repo.identity_doc()?.visibility.is_private() {
        return Err(Error::NotFound);
    }

    let blob = repo.blob(oid)?;
    let mut response_headers = HeaderMap::new();

    if blob.size() > MAX_BLOB_SIZE {
        return Ok::<_, Error>((StatusCode::PAYLOAD_TOO_LARGE, response_headers, vec![]));
    }

    response_headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&qs.mime.unwrap_or("application/octet-stream".to_string()))?,
    );

    Ok::<_, Error>((StatusCode::OK, response_headers, blob.content().to_vec()))
}

#[cfg(test)]
mod routes {
    use axum::http::StatusCode;

    use crate::test::{self, get, RID, RID_PRIVATE};
    use radicle::storage::ReadStorage;

    #[tokio::test]
    async fn test_file_handler() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = test::seed(tmp.path());
        let app = super::router(ctx.profile().to_owned());

        let response = get(&app, format!("/{RID}/head/dir1/README")).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.body().await, "Hello World from dir1!\n");

        // Make sure the repo exists in storage.
        ctx.profile()
            .storage
            .repository(RID_PRIVATE.parse().unwrap())
            .unwrap();

        let response = get(&app, format!("/{RID_PRIVATE}/head/README")).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
