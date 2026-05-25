use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::AppContext;
use crate::domain::{
    Item, ItemMedia, Location, NewItem, NewItemMediaInput, NewLocation, UpdateItemInput,
};
use crate::error::AppError;

#[derive(Clone)]
struct ApiState {
    context: AppContext,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    item_count: i64,
    location_count: i64,
    trip_count: i64,
}

#[derive(Debug, Serialize)]
struct ServerInfoResponse {
    application: &'static str,
    version: &'static str,
    bind_url: String,
    local_url: String,
    lan_url: Option<String>,
    data_dir: String,
    database_file: String,
}

#[derive(Debug, Serialize)]
struct ItemsListResponse {
    items: Vec<ItemResponse>,
}

#[derive(Debug, Serialize)]
struct LocationsListResponse {
    locations: Vec<LocationResponse>,
}

#[derive(Debug, Serialize)]
struct ItemMediaListResponse {
    media: Vec<ItemMediaResponse>,
}

#[derive(Debug, Serialize)]
struct ItemResponse {
    id: String,
    name: String,
    category: Option<String>,
    subcategory: Option<String>,
    brand: Option<String>,
    size: Option<String>,
    color_primary: Option<String>,
    color_secondary: Option<String>,
    material: Option<String>,
    season: Option<String>,
    formality: Option<String>,
    status: Option<String>,
    current_location_id: Option<String>,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct ItemMediaResponse {
    id: String,
    item_id: String,
    media_kind: String,
    relative_file_path: String,
    original_filename: String,
    mime_type: String,
    file_size_bytes: i64,
    duration_ms: Option<i64>,
    width: Option<i64>,
    height: Option<i64>,
    caption: Option<String>,
    sort_order: i64,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct LocationResponse {
    id: String,
    name: String,
    location_type: String,
    parent_id: Option<String>,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct CreateItemRequest {
    name: String,
    category: Option<String>,
    subcategory: Option<String>,
    brand: Option<String>,
    size: Option<String>,
    color_primary: Option<String>,
    color_secondary: Option<String>,
    material: Option<String>,
    season: Option<String>,
    formality: Option<String>,
    status: Option<String>,
    current_location_id: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateLocationRequest {
    name: String,
    location_type: String,
    parent_id: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PatchItemRequest {
    name: Option<String>,
    category: Option<String>,
    subcategory: Option<String>,
    brand: Option<String>,
    size: Option<String>,
    color_primary: Option<String>,
    color_secondary: Option<String>,
    material: Option<String>,
    season: Option<String>,
    formality: Option<String>,
    status: Option<String>,
    current_location_id: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Value>,
}

struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
    details: Option<Value>,
}

pub fn router(context: AppContext) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/server-info", get(server_info_handler))
        .route("/items", get(list_items_handler).post(create_item_handler))
        .route(
            "/items/{id}",
            get(get_item_handler).patch(update_item_handler),
        )
        .route(
            "/items/{id}/media",
            get(list_item_media_handler).post(upload_item_media_handler),
        )
        .route(
            "/locations",
            get(list_locations_handler).post(create_location_handler),
        )
        .route("/locations/{id}", get(get_location_handler))
        .with_state(ApiState { context })
}

async fn health_handler(State(state): State<ApiState>) -> Result<Json<HealthResponse>, ApiError> {
    let health = state
        .context
        .service
        .health()
        .await
        .map_err(ApiError::from)?;

    Ok(Json(HealthResponse {
        status: "ok",
        item_count: health.item_count,
        location_count: health.location_count,
        trip_count: health.trip_count,
    }))
}

async fn server_info_handler(
    State(state): State<ApiState>,
) -> Result<Json<ServerInfoResponse>, ApiError> {
    Ok(Json(ServerInfoResponse {
        application: "MyWardrobeHelper",
        version: env!("CARGO_PKG_VERSION"),
        bind_url: state.context.config.bind_url(),
        local_url: state.context.config.local_url(),
        lan_url: state.context.config.lan_url(),
        data_dir: state.context.layout.root.display().to_string(),
        database_file: state.context.layout.database_file.display().to_string(),
    }))
}

async fn list_items_handler(
    State(state): State<ApiState>,
) -> Result<Json<ItemsListResponse>, ApiError> {
    let items = state
        .context
        .service
        .list_items()
        .await
        .map_err(ApiError::from)?;

    Ok(Json(ItemsListResponse {
        items: items.into_iter().map(ItemResponse::from).collect(),
    }))
}

async fn create_item_handler(
    State(state): State<ApiState>,
    Json(request): Json<CreateItemRequest>,
) -> Result<(StatusCode, Json<ItemResponse>), ApiError> {
    let item = state
        .context
        .service
        .create_item(NewItem {
            name: request.name,
            category: request.category,
            subcategory: request.subcategory,
            brand: request.brand,
            size: request.size,
            color_primary: request.color_primary,
            color_secondary: request.color_secondary,
            material: request.material,
            season: request.season,
            formality: request.formality,
            status: request.status,
            current_location_id: request.current_location_id,
            notes: request.notes,
        })
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(ItemResponse::from(item))))
}

async fn get_item_handler(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ItemResponse>, ApiError> {
    let item = state
        .context
        .service
        .get_item(&id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| {
            ApiError::not_found(
                "ITEM_NOT_FOUND",
                "Item not found",
                Some(json!({ "item_id": id })),
            )
        })?;

    Ok(Json(ItemResponse::from(item)))
}

async fn update_item_handler(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(request): Json<PatchItemRequest>,
) -> Result<Json<ItemResponse>, ApiError> {
    let existing = state
        .context
        .service
        .get_item(&id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| {
            ApiError::not_found(
                "ITEM_NOT_FOUND",
                "Item not found",
                Some(json!({ "item_id": id.clone() })),
            )
        })?;

    let updated = state
        .context
        .service
        .update_item(
            &id,
            UpdateItemInput {
                name: request.name.unwrap_or(existing.name),
                category: request.category.or(existing.category),
                subcategory: request.subcategory.or(existing.subcategory),
                brand: request.brand.or(existing.brand),
                size: request.size.or(existing.size),
                color_primary: request.color_primary.or(existing.color_primary),
                color_secondary: request.color_secondary.or(existing.color_secondary),
                material: request.material.or(existing.material),
                season: request.season.or(existing.season),
                formality: request.formality.or(existing.formality),
                status: request.status.or(existing.status),
                current_location_id: request.current_location_id.or(existing.current_location_id),
                notes: request.notes.or(existing.notes),
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Json(ItemResponse::from(updated)))
}

async fn list_item_media_handler(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ItemMediaListResponse>, ApiError> {
    let media = state
        .context
        .service
        .list_item_media(&id)
        .await
        .map_err(item_related_error(&id))?;

    Ok(Json(ItemMediaListResponse {
        media: media.into_iter().map(ItemMediaResponse::from).collect(),
    }))
}

async fn upload_item_media_handler(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ItemMediaListResponse>), ApiError> {
    let mut caption: Option<String> = None;
    let mut uploads = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|_| {
        ApiError::bad_request(
            "INVALID_MULTIPART",
            "Multipart upload could not be parsed".to_string(),
            None,
        )
    })? {
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name == "caption" {
            caption = field.text().await.ok();
            continue;
        }

        if field_name != "file" {
            continue;
        }

        let mime_type = field
            .content_type()
            .map(str::to_string)
            .unwrap_or_else(|| "application/octet-stream".to_string());
        let original_filename = field.file_name().map(str::to_string);
        let bytes = field.bytes().await.map_err(|_| {
            ApiError::bad_request(
                "INVALID_MULTIPART",
                "Uploaded media bytes could not be read".to_string(),
                None,
            )
        })?;

        if bytes.is_empty() {
            continue;
        }

        uploads.push(NewItemMediaInput {
            media_kind: String::new(),
            original_filename,
            mime_type,
            caption: caption.clone(),
            bytes: bytes.to_vec(),
        });
    }

    if uploads.is_empty() {
        return Err(ApiError::bad_request(
            "NO_MEDIA_FILES",
            "At least one image or video file is required".to_string(),
            None,
        ));
    }

    let mut created = Vec::new();
    for upload in uploads {
        let media = state
            .context
            .service
            .attach_item_media(&id, upload)
            .await
            .map_err(item_related_error(&id))?;
        created.push(ItemMediaResponse::from(media));
    }

    Ok((
        StatusCode::CREATED,
        Json(ItemMediaListResponse { media: created }),
    ))
}

async fn list_locations_handler(
    State(state): State<ApiState>,
) -> Result<Json<LocationsListResponse>, ApiError> {
    let locations = state
        .context
        .service
        .list_locations()
        .await
        .map_err(ApiError::from)?;

    Ok(Json(LocationsListResponse {
        locations: locations.into_iter().map(LocationResponse::from).collect(),
    }))
}

async fn create_location_handler(
    State(state): State<ApiState>,
    Json(request): Json<CreateLocationRequest>,
) -> Result<(StatusCode, Json<LocationResponse>), ApiError> {
    let location = state
        .context
        .service
        .create_location(NewLocation {
            name: request.name,
            location_type: request.location_type,
            parent_id: request.parent_id,
            notes: request.notes,
        })
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(LocationResponse::from(location))))
}

async fn get_location_handler(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<LocationResponse>, ApiError> {
    let location = state
        .context
        .service
        .get_location(&id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| {
            ApiError::not_found(
                "LOCATION_NOT_FOUND",
                "Location not found",
                Some(json!({ "location_id": id })),
            )
        })?;

    Ok(Json(LocationResponse::from(location)))
}

impl ItemResponse {
    fn from_domain(item: Item) -> Self {
        Self {
            id: item.id,
            name: item.name,
            category: item.category,
            subcategory: item.subcategory,
            brand: item.brand,
            size: item.size,
            color_primary: item.color_primary,
            color_secondary: item.color_secondary,
            material: item.material,
            season: item.season,
            formality: item.formality,
            status: item.status,
            current_location_id: item.current_location_id,
            notes: item.notes,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

impl From<Item> for ItemResponse {
    fn from(value: Item) -> Self {
        Self::from_domain(value)
    }
}

impl From<ItemMedia> for ItemMediaResponse {
    fn from(value: ItemMedia) -> Self {
        Self {
            id: value.id,
            item_id: value.item_id,
            media_kind: value.media_kind,
            relative_file_path: value.relative_file_path,
            original_filename: value.original_filename,
            mime_type: value.mime_type,
            file_size_bytes: value.file_size_bytes,
            duration_ms: value.duration_ms,
            width: value.width,
            height: value.height,
            caption: value.caption,
            sort_order: value.sort_order,
            created_at: value.created_at,
        }
    }
}

impl From<Location> for LocationResponse {
    fn from(value: Location) -> Self {
        Self {
            id: value.id,
            name: value.name,
            location_type: value.location_type,
            parent_id: value.parent_id,
            notes: value.notes,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl ApiError {
    fn bad_request(code: &'static str, message: String, details: Option<Value>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message,
            details,
        }
    }

    fn internal(code: &'static str, message: &'static str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code,
            message: message.to_string(),
            details: None,
        }
    }

    fn not_found(code: &'static str, message: &'static str, details: Option<Value>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code,
            message: message.to_string(),
            details,
        }
    }
}

fn item_related_error(item_id: &str) -> impl Fn(AppError) -> ApiError + '_ {
    move |error| match error {
        AppError::InvalidArgument(message) if message.contains("does not exist") => {
            ApiError::not_found(
                "ITEM_NOT_FOUND",
                "Item not found",
                Some(json!({ "item_id": item_id })),
            )
        }
        other => ApiError::from(other),
    }
}

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        match value {
            AppError::InvalidArgument(message) => {
                Self::bad_request("INVALID_REQUEST", message, None)
            }
            AppError::NotInitialized { .. } => Self {
                status: StatusCode::SERVICE_UNAVAILABLE,
                code: "SERVICE_NOT_READY",
                message: "The local wardrobe backend is not initialized yet.".to_string(),
                details: None,
            },
            AppError::Config(_) | AppError::Io { .. } | AppError::Database { .. } => {
                Self::internal("INTERNAL_ERROR", "Internal server error")
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorEnvelope {
                error: ErrorBody {
                    code: self.code,
                    message: self.message,
                    details: self.details,
                },
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use crate::app::{init_app, open_context};
    use crate::config::{AppConfig, DEFAULT_HOST, DEFAULT_PORT};

    use super::*;

    #[tokio::test]
    async fn health_endpoint_returns_counts() {
        let sandbox = ApiSandbox::new();
        let app = router(sandbox.context().await);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("health response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_json(response.into_body()).await;
        assert_eq!(body["status"], "ok");
        assert_eq!(body["item_count"], 0);
    }

    #[tokio::test]
    async fn server_info_endpoint_returns_runtime_details() {
        let sandbox = ApiSandbox::new();
        let app = router(sandbox.context().await);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/server-info")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("server-info response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_json(response.into_body()).await;
        assert_eq!(body["application"], "MyWardrobeHelper");
        assert!(
            body["data_dir"]
                .as_str()
                .unwrap()
                .contains("mywardrobehelper-api-test")
        );
    }

    #[tokio::test]
    async fn item_routes_support_create_list_and_get() {
        let sandbox = ApiSandbox::new();
        let app = router(sandbox.context().await);

        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/items")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "name": "Travel Blazer",
                            "category": "Outerwear",
                            "brand": "Example"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("create item response");

        assert_eq!(create_response.status(), StatusCode::CREATED);
        let created_item = to_json(create_response.into_body()).await;
        let item_id = created_item["id"].as_str().unwrap().to_string();

        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/items")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("list items response");
        assert_eq!(list_response.status(), StatusCode::OK);
        let list_body = to_json(list_response.into_body()).await;
        assert_eq!(list_body["items"][0]["name"], "Travel Blazer");

        let get_response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/items/{item_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("get item response");
        assert_eq!(get_response.status(), StatusCode::OK);
        let get_body = to_json(get_response.into_body()).await;
        assert_eq!(get_body["id"], item_id);
    }

    #[tokio::test]
    async fn item_patch_updates_existing_record() {
        let sandbox = ApiSandbox::new();
        let app = router(sandbox.context().await);

        let created = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/items")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({ "name": "Wool Coat", "category": "Outerwear" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("create item response");
        let created_body = to_json(created.into_body()).await;
        let item_id = created_body["id"].as_str().unwrap();

        let patched = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/items/{item_id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({ "brand": "Example", "status": "ready" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("patch item response");

        assert_eq!(patched.status(), StatusCode::OK);
        let patched_body = to_json(patched.into_body()).await;
        assert_eq!(patched_body["brand"], "Example");
        assert_eq!(patched_body["status"], "ready");
    }

    #[tokio::test]
    async fn item_media_routes_support_upload_and_list() {
        let sandbox = ApiSandbox::new();
        let app = router(sandbox.context().await);

        let created = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/items")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({ "name": "Travel Coat" }).to_string()))
                    .unwrap(),
            )
            .await
            .expect("create item response");
        let created_body = to_json(created.into_body()).await;
        let item_id = created_body["id"].as_str().unwrap().to_string();

        let boundary = "mwh-boundary";
        let multipart_body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"caption\"\r\n\r\nFront view\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"coat.jpg\"\r\nContent-Type: image/jpeg\r\n\r\nfake-image-bytes\r\n--{boundary}--\r\n"
        );

        let upload = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/media"))
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(multipart_body))
                    .unwrap(),
            )
            .await
            .expect("upload media response");

        assert_eq!(upload.status(), StatusCode::CREATED);
        let upload_body = to_json(upload.into_body()).await;
        assert_eq!(upload_body["media"][0]["media_kind"], "image");

        let listed = app
            .oneshot(
                Request::builder()
                    .uri(format!("/items/{item_id}/media"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("list media response");

        assert_eq!(listed.status(), StatusCode::OK);
        let listed_body = to_json(listed.into_body()).await;
        assert_eq!(listed_body["media"][0]["caption"], "Front view");
        assert!(
            listed_body["media"][0]["relative_file_path"]
                .as_str()
                .unwrap()
                .contains(&item_id)
        );
    }

    #[tokio::test]
    async fn location_routes_support_create_list_and_get() {
        let sandbox = ApiSandbox::new();
        let app = router(sandbox.context().await);

        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/locations")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "name": "Hall Closet",
                            "location_type": "Closet"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("create location response");

        assert_eq!(create_response.status(), StatusCode::CREATED);
        let created_location = to_json(create_response.into_body()).await;
        let location_id = created_location["id"].as_str().unwrap().to_string();

        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/locations")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("list locations response");
        assert_eq!(list_response.status(), StatusCode::OK);
        let list_body = to_json(list_response.into_body()).await;
        assert_eq!(list_body["locations"][0]["name"], "Hall Closet");

        let get_response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/locations/{location_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("get location response");
        assert_eq!(get_response.status(), StatusCode::OK);
        let get_body = to_json(get_response.into_body()).await;
        assert_eq!(get_body["id"], location_id);
    }

    #[tokio::test]
    async fn missing_item_uses_structured_error_shape() {
        let sandbox = ApiSandbox::new();
        let app = router(sandbox.context().await);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items/does-not-exist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("missing item response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = to_json(response.into_body()).await;
        assert_eq!(body["error"]["code"], "ITEM_NOT_FOUND");
        assert_eq!(body["error"]["message"], "Item not found");
        assert_eq!(body["error"]["details"]["item_id"], "does-not-exist");
    }

    async fn to_json(body: Body) -> Value {
        let bytes = to_bytes(body, usize::MAX).await.expect("read body");
        serde_json::from_slice(&bytes).expect("valid json")
    }

    struct ApiSandbox {
        root: PathBuf,
        data_dir: PathBuf,
    }

    impl ApiSandbox {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);

            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let root = env::temp_dir().join(format!(
                "mywardrobehelper-api-test-{}-{}",
                std::process::id(),
                unique
            ));

            Self {
                data_dir: root.join("data"),
                root,
            }
        }

        async fn context(&self) -> AppContext {
            let config = AppConfig {
                host: DEFAULT_HOST.to_string(),
                port: DEFAULT_PORT,
                data_dir: self.data_dir.clone(),
            };

            init_app(&config).await.expect("initialize database");
            open_context(config).await.expect("open app context")
        }
    }

    impl Drop for ApiSandbox {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
