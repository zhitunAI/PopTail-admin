use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

use super::{
    resume_upload_advance, resume_upload_list, resume_upload_resume, scan_session_get,
    scan_session_update, upload_file_asset, upload_queue_complete, upload_queue_replace,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/fileUploadAndDownload/upload", post(upload_file_asset))
        .route("/example/upload/list", post(upload_queue_replace))
        .route("/example/upload/complete", post(upload_queue_complete))
        .route("/example/resume/list", get(resume_upload_list))
        .route("/example/resume/advance", post(resume_upload_advance))
        .route("/example/resume/recover", post(resume_upload_resume))
        .route("/example/scan/session", get(scan_session_get))
        .route("/example/scan/session", post(scan_session_update))
}
