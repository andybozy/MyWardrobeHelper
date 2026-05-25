use askama::Template;
use axum::Router;
use axum::extract::{Form, Multipart, Path, State};
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{get, post};
use mime_guess::MimeGuess;
use serde::Deserialize;
use tokio::net::TcpListener;

use crate::api;
use crate::app::{self, AppContext};
use crate::domain::{
    HealthSnapshot, Item, ItemMedia, Location, NewItem, NewItemMediaInput, Trip, UpdateItemInput,
};
use crate::error::{AppError, AppResult};
use crate::infra::MediaStorage;

#[derive(Clone)]
struct WebState {
    context: AppContext,
}

#[derive(Debug, Clone)]
struct StatCard {
    label: &'static str,
    value: String,
    detail: String,
}

#[derive(Debug, Clone)]
struct ListEntry {
    title: String,
    subtitle: String,
    meta: String,
}

#[derive(Debug, Clone)]
struct StatusCheckView {
    label: &'static str,
    badge: &'static str,
    badge_class: &'static str,
    message: String,
}

#[derive(Debug, Clone)]
struct ItemField {
    label: &'static str,
    value: String,
}

#[derive(Debug, Clone)]
struct MediaView {
    url: String,
    media_kind: String,
    original_filename: String,
    caption: String,
    mime_type: String,
    file_size_label: String,
}

#[derive(Debug, Clone)]
struct ItemDetailView {
    id: String,
    name: String,
    edit_url: String,
    upload_url: String,
}

#[derive(Debug, Clone)]
struct ItemFormView {
    name: String,
    category: String,
    subcategory: String,
    brand: String,
    size: String,
    color_primary: String,
    color_secondary: String,
    material: String,
    season: String,
    formality: String,
    status: String,
    current_location_id: String,
    notes: String,
}

#[derive(Debug, Deserialize)]
struct ItemFormData {
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

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    page_title: &'static str,
    nav_home_active: bool,
    nav_items_active: bool,
    nav_status_active: bool,
    data_dir: String,
    local_url: String,
    lan_url: Option<String>,
    stats: Vec<StatCard>,
    has_recent_items: bool,
    has_recent_locations: bool,
    has_recent_trips: bool,
    recent_items: Vec<ListEntry>,
    recent_locations: Vec<ListEntry>,
    recent_trips: Vec<ListEntry>,
}

#[derive(Template)]
#[template(path = "status.html")]
struct StatusTemplate {
    page_title: &'static str,
    nav_home_active: bool,
    nav_items_active: bool,
    nav_status_active: bool,
    data_dir: String,
    bind_url: String,
    local_url: String,
    lan_url: Option<String>,
    database_file: String,
    health: HealthSnapshot,
    checks: Vec<StatusCheckView>,
}

#[derive(Template)]
#[template(path = "items.html")]
struct ItemsTemplate {
    page_title: &'static str,
    nav_home_active: bool,
    nav_items_active: bool,
    nav_status_active: bool,
    data_dir: String,
    has_items: bool,
    items: Vec<ListEntry>,
}

#[derive(Template)]
#[template(path = "item_form.html")]
struct ItemFormTemplate {
    page_title: &'static str,
    nav_home_active: bool,
    nav_items_active: bool,
    nav_status_active: bool,
    data_dir: String,
    heading: &'static str,
    submit_label: &'static str,
    action_url: String,
    item: ItemFormView,
}

#[derive(Template)]
#[template(path = "item_detail.html")]
struct ItemDetailTemplate {
    page_title: String,
    nav_home_active: bool,
    nav_items_active: bool,
    nav_status_active: bool,
    data_dir: String,
    item: ItemDetailView,
    fields: Vec<ItemField>,
    has_media: bool,
    media: Vec<MediaView>,
}

pub async fn serve(context: AppContext) -> AppResult<()> {
    let bind_host = context.config.host.clone();
    let port = context.config.port;
    let bind_url = context.config.bind_url();
    let local_url = context.config.local_url();
    let lan_url = context.config.lan_url();
    let data_dir = context.layout.root.display().to_string();

    let router = router(context);
    let listener = TcpListener::bind((bind_host.as_str(), port))
        .await
        .map_err(|error| AppError::io(format!("bind HTTP listener at {bind_url}"), error))?;

    println!("Serving MyWardrobeHelper");
    println!("Bind URL: {bind_url}");
    println!("Local UI: {local_url}");
    match &lan_url {
        Some(url) => println!("LAN UI: {url}"),
        None => println!("LAN UI: disabled (bind host is loopback only)"),
    }
    println!("Data directory: {data_dir}");
    println!("Press Ctrl-C to stop the local server.");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|error| AppError::io("run HTTP server", error))
}

fn router(context: AppContext) -> Router {
    Router::new()
        .route("/", get(home_handler))
        .route("/items", get(items_list_handler).post(item_create_handler))
        .route("/items/new", get(item_new_handler))
        .route("/items/{id}", get(item_detail_handler))
        .route(
            "/items/{id}/edit",
            get(item_edit_handler).post(item_update_handler),
        )
        .route("/items/{id}/media", post(item_media_upload_handler))
        .route("/status", get(status_handler))
        .route("/assets/app.css", get(stylesheet_handler))
        .route("/media/{*path}", get(media_file_handler))
        .with_state(WebState {
            context: context.clone(),
        })
        .nest("/api/v1", api::router(context))
}

async fn home_handler(State(state): State<WebState>) -> Result<Html<String>, StatusCode> {
    let health = state
        .context
        .service
        .health()
        .await
        .map_err(internal_error_status)?;
    let recent_items = state
        .context
        .service
        .list_items()
        .await
        .map_err(internal_error_status)?;
    let recent_locations = state
        .context
        .service
        .list_locations()
        .await
        .map_err(internal_error_status)?;
    let recent_trips = state
        .context
        .service
        .list_trips()
        .await
        .map_err(internal_error_status)?;

    let template = HomeTemplate {
        page_title: "Dashboard",
        nav_home_active: true,
        nav_items_active: false,
        nav_status_active: false,
        data_dir: state.context.layout.root.display().to_string(),
        local_url: state.context.config.local_url(),
        lan_url: state.context.config.lan_url(),
        stats: vec![
            StatCard {
                label: "Items",
                value: health.item_count.to_string(),
                detail: "Tracked wardrobe pieces".to_string(),
            },
            StatCard {
                label: "Locations",
                value: health.location_count.to_string(),
                detail: "Storage places and containers".to_string(),
            },
            StatCard {
                label: "Trips",
                value: health.trip_count.to_string(),
                detail: "Packing plans in the database".to_string(),
            },
        ],
        has_recent_items: !recent_items.is_empty(),
        has_recent_locations: !recent_locations.is_empty(),
        has_recent_trips: !recent_trips.is_empty(),
        recent_items: recent_items
            .into_iter()
            .rev()
            .take(5)
            .map(item_entry)
            .collect(),
        recent_locations: recent_locations
            .into_iter()
            .rev()
            .take(5)
            .map(location_entry)
            .collect(),
        recent_trips: recent_trips
            .into_iter()
            .rev()
            .take(5)
            .map(trip_entry)
            .collect(),
    };

    render_template(&template)
}

async fn items_list_handler(State(state): State<WebState>) -> Result<Html<String>, StatusCode> {
    let items = state
        .context
        .service
        .list_items()
        .await
        .map_err(internal_error_status)?;

    let template = ItemsTemplate {
        page_title: "Items",
        nav_home_active: false,
        nav_items_active: true,
        nav_status_active: false,
        data_dir: state.context.layout.root.display().to_string(),
        has_items: !items.is_empty(),
        items: items
            .into_iter()
            .map(|item| ListEntry {
                title: item.name,
                subtitle: join_optional_parts([
                    item.category.as_deref(),
                    item.brand.as_deref(),
                    item.size.as_deref(),
                ]),
                meta: format!("/items/{}", item.id),
            })
            .collect(),
    };

    render_template(&template)
}

async fn item_new_handler(State(state): State<WebState>) -> Result<Html<String>, StatusCode> {
    let template = ItemFormTemplate {
        page_title: "New Item",
        nav_home_active: false,
        nav_items_active: true,
        nav_status_active: false,
        data_dir: state.context.layout.root.display().to_string(),
        heading: "Create Item",
        submit_label: "Create item",
        action_url: "/items".to_string(),
        item: empty_item_form(),
    };

    render_template(&template)
}

async fn item_create_handler(
    State(state): State<WebState>,
    Form(form): Form<ItemFormData>,
) -> Result<Redirect, StatusCode> {
    let item = state
        .context
        .service
        .create_item(NewItem {
            name: form.name,
            category: form.category,
            subcategory: form.subcategory,
            brand: form.brand,
            size: form.size,
            color_primary: form.color_primary,
            color_secondary: form.color_secondary,
            material: form.material,
            season: form.season,
            formality: form.formality,
            status: form.status,
            current_location_id: form.current_location_id,
            notes: form.notes,
        })
        .await
        .map_err(internal_error_status)?;

    Ok(Redirect::to(&format!("/items/{}", item.id)))
}

async fn item_detail_handler(
    State(state): State<WebState>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let item = state
        .context
        .service
        .get_item(&id)
        .await
        .map_err(internal_error_status)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let media = state
        .context
        .service
        .list_item_media(&id)
        .await
        .map_err(internal_error_status)?;

    let template = ItemDetailTemplate {
        page_title: format!("Item · {}", item.name),
        nav_home_active: false,
        nav_items_active: true,
        nav_status_active: false,
        data_dir: state.context.layout.root.display().to_string(),
        item: ItemDetailView {
            id: item.id.clone(),
            name: item.name.clone(),
            edit_url: format!("/items/{}/edit", item.id),
            upload_url: format!("/items/{}/media", item.id),
        },
        fields: item_fields(&item),
        has_media: !media.is_empty(),
        media: media.into_iter().map(media_view).collect(),
    };

    render_template(&template)
}

async fn item_edit_handler(
    State(state): State<WebState>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let item = state
        .context
        .service
        .get_item(&id)
        .await
        .map_err(internal_error_status)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let template = ItemFormTemplate {
        page_title: "Edit Item",
        nav_home_active: false,
        nav_items_active: true,
        nav_status_active: false,
        data_dir: state.context.layout.root.display().to_string(),
        heading: "Edit Item",
        submit_label: "Save changes",
        action_url: format!("/items/{}/edit", item.id),
        item: item_form_view(&item),
    };

    render_template(&template)
}

async fn item_update_handler(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(form): Form<ItemFormData>,
) -> Result<Redirect, StatusCode> {
    state
        .context
        .service
        .update_item(
            &id,
            UpdateItemInput {
                name: form.name,
                category: form.category,
                subcategory: form.subcategory,
                brand: form.brand,
                size: form.size,
                color_primary: form.color_primary,
                color_secondary: form.color_secondary,
                material: form.material,
                season: form.season,
                formality: form.formality,
                status: form.status,
                current_location_id: form.current_location_id,
                notes: form.notes,
            },
        )
        .await
        .map_err(internal_error_status)?;

    Ok(Redirect::to(&format!("/items/{id}")))
}

async fn item_media_upload_handler(
    State(state): State<WebState>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<Redirect, StatusCode> {
    let mut caption: Option<String> = None;
    let mut uploads = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
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
        let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
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
        return Err(StatusCode::BAD_REQUEST);
    }

    for upload in uploads {
        state
            .context
            .service
            .attach_item_media(&id, upload)
            .await
            .map_err(internal_error_status)?;
    }

    Ok(Redirect::to(&format!("/items/{id}")))
}

async fn status_handler(State(state): State<WebState>) -> Result<Html<String>, StatusCode> {
    let health = state
        .context
        .service
        .health()
        .await
        .map_err(internal_error_status)?;
    let report = app::doctor(&state.context.config).await;

    let template = StatusTemplate {
        page_title: "System Status",
        nav_home_active: false,
        nav_items_active: false,
        nav_status_active: true,
        data_dir: state.context.layout.root.display().to_string(),
        bind_url: state.context.config.bind_url(),
        local_url: state.context.config.local_url(),
        lan_url: state.context.config.lan_url(),
        database_file: state.context.layout.database_file.display().to_string(),
        health,
        checks: report
            .checks
            .into_iter()
            .map(|check| StatusCheckView {
                label: check.label,
                badge: match check.status {
                    app::CheckStatus::Pass => "PASS",
                    app::CheckStatus::Warn => "WARN",
                    app::CheckStatus::Fail => "FAIL",
                },
                badge_class: match check.status {
                    app::CheckStatus::Pass => "is-pass",
                    app::CheckStatus::Warn => "is-warn",
                    app::CheckStatus::Fail => "is-fail",
                },
                message: check.message,
            })
            .collect(),
    };

    render_template(&template)
}

async fn stylesheet_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../../assets/app.css"),
    )
}

async fn media_file_handler(
    State(state): State<WebState>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let storage = MediaStorage::new(state.context.layout.root.clone());
    let relative_path = format!("media/{path}");
    let bytes = storage
        .read_relative(&relative_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let content_type = MimeGuess::from_path(&relative_path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    Ok(([(header::CONTENT_TYPE, content_type)], bytes))
}

fn render_template<T: Template>(template: &T) -> Result<Html<String>, StatusCode> {
    template
        .render()
        .map(Html)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn item_entry(item: Item) -> ListEntry {
    ListEntry {
        title: item.name,
        subtitle: join_optional_parts([
            item.category.as_deref(),
            item.brand.as_deref(),
            item.size.as_deref(),
        ]),
        meta: item
            .status
            .unwrap_or_else(|| "No status set yet".to_string()),
    }
}

fn location_entry(location: Location) -> ListEntry {
    ListEntry {
        title: location.name,
        subtitle: location.location_type,
        meta: match location.parent_id {
            Some(parent_id) => format!("Nested under {parent_id}"),
            None => "Top-level location".to_string(),
        },
    }
}

fn trip_entry(trip: Trip) -> ListEntry {
    ListEntry {
        title: trip.name,
        subtitle: join_optional_parts([trip.destination.as_deref(), trip.trip_type.as_deref()]),
        meta: join_optional_parts([trip.start_date.as_deref(), trip.end_date.as_deref()]),
    }
}

fn item_fields(item: &Item) -> Vec<ItemField> {
    vec![
        field("Category", item.category.as_deref()),
        field("Subcategory", item.subcategory.as_deref()),
        field("Brand", item.brand.as_deref()),
        field("Size", item.size.as_deref()),
        field("Primary color", item.color_primary.as_deref()),
        field("Secondary color", item.color_secondary.as_deref()),
        field("Material", item.material.as_deref()),
        field("Season", item.season.as_deref()),
        field("Formality", item.formality.as_deref()),
        field("Status", item.status.as_deref()),
        field("Current location", item.current_location_id.as_deref()),
        field("Notes", item.notes.as_deref()),
        ItemField {
            label: "Created",
            value: item.created_at.clone(),
        },
        ItemField {
            label: "Updated",
            value: item.updated_at.clone(),
        },
    ]
}

fn field(label: &'static str, value: Option<&str>) -> ItemField {
    ItemField {
        label,
        value: value
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("Not set")
            .to_string(),
    }
}

fn item_form_view(item: &Item) -> ItemFormView {
    ItemFormView {
        name: item.name.clone(),
        category: item.category.clone().unwrap_or_default(),
        subcategory: item.subcategory.clone().unwrap_or_default(),
        brand: item.brand.clone().unwrap_or_default(),
        size: item.size.clone().unwrap_or_default(),
        color_primary: item.color_primary.clone().unwrap_or_default(),
        color_secondary: item.color_secondary.clone().unwrap_or_default(),
        material: item.material.clone().unwrap_or_default(),
        season: item.season.clone().unwrap_or_default(),
        formality: item.formality.clone().unwrap_or_default(),
        status: item.status.clone().unwrap_or_default(),
        current_location_id: item.current_location_id.clone().unwrap_or_default(),
        notes: item.notes.clone().unwrap_or_default(),
    }
}

fn empty_item_form() -> ItemFormView {
    ItemFormView {
        name: String::new(),
        category: String::new(),
        subcategory: String::new(),
        brand: String::new(),
        size: String::new(),
        color_primary: String::new(),
        color_secondary: String::new(),
        material: String::new(),
        season: String::new(),
        formality: String::new(),
        status: String::new(),
        current_location_id: String::new(),
        notes: String::new(),
    }
}

fn media_view(media: ItemMedia) -> MediaView {
    MediaView {
        url: format!("/{}", media.relative_file_path),
        media_kind: media.media_kind,
        original_filename: media.original_filename,
        caption: media.caption.unwrap_or_default(),
        mime_type: media.mime_type,
        file_size_label: format!("{} bytes", media.file_size_bytes),
    }
}

fn join_optional_parts<'a>(parts: impl IntoIterator<Item = Option<&'a str>>) -> String {
    let values: Vec<&str> = parts
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect();

    if values.is_empty() {
        "No extra details yet".to_string()
    } else {
        values.join(" · ")
    }
}

fn internal_error_status(_error: AppError) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use tower::ServiceExt;

    use crate::app::{init_app, open_context};
    use crate::config::{AppConfig, DEFAULT_HOST, DEFAULT_PORT};
    use crate::domain::{NewItem, NewItemMediaInput, NewLocation, NewTrip};

    use super::*;

    #[tokio::test]
    async fn home_page_renders_dashboard() {
        let sandbox = WebSandbox::new();
        let context = sandbox.context().await;

        context
            .service
            .create_item(NewItem {
                name: "Field Jacket".to_string(),
                category: Some("Outerwear".to_string()),
                subcategory: None,
                brand: Some("Archive".to_string()),
                size: Some("L".to_string()),
                color_primary: None,
                color_secondary: None,
                material: None,
                season: None,
                formality: None,
                status: Some("ready".to_string()),
                current_location_id: None,
                notes: None,
            })
            .await
            .expect("create sample item");

        let app = router(context);
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .expect("home route response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read body");
        let html = String::from_utf8(body.to_vec()).expect("valid utf-8 html");

        assert!(html.contains("Wardrobe Dashboard"));
        assert!(html.contains("Field Jacket"));
        assert!(html.contains("Recent Items"));
    }

    #[tokio::test]
    async fn item_detail_page_renders_media_gallery() {
        let sandbox = WebSandbox::new();
        let context = sandbox.context().await;
        let item = context
            .service
            .create_item(NewItem {
                name: "Travel Sweater".to_string(),
                category: None,
                subcategory: None,
                brand: None,
                size: None,
                color_primary: None,
                color_secondary: None,
                material: None,
                season: None,
                formality: None,
                status: None,
                current_location_id: None,
                notes: None,
            })
            .await
            .expect("create sample item");
        context
            .service
            .attach_item_media(
                &item.id,
                NewItemMediaInput {
                    media_kind: String::new(),
                    original_filename: Some("coat.jpg".to_string()),
                    mime_type: "image/jpeg".to_string(),
                    caption: Some("Front view".to_string()),
                    bytes: b"fake-image-bytes".to_vec(),
                },
            )
            .await
            .expect("attach sample media");

        let app = router(context);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/items/{}", item.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("item detail response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read body");
        let html = String::from_utf8(body.to_vec()).expect("valid utf-8 html");

        assert!(html.contains("Travel Sweater"));
        assert!(html.contains("Front view"));
        assert!(html.contains("Media Gallery"));
    }

    #[tokio::test]
    async fn media_route_serves_uploaded_bytes() {
        let sandbox = WebSandbox::new();
        let context = sandbox.context().await;
        let item = context
            .service
            .create_item(NewItem {
                name: "Trail Vest".to_string(),
                category: None,
                subcategory: None,
                brand: None,
                size: None,
                color_primary: None,
                color_secondary: None,
                material: None,
                season: None,
                formality: None,
                status: None,
                current_location_id: None,
                notes: None,
            })
            .await
            .expect("create sample item");
        let media = context
            .service
            .attach_item_media(
                &item.id,
                NewItemMediaInput {
                    media_kind: String::new(),
                    original_filename: Some("vest.jpg".to_string()),
                    mime_type: "image/jpeg".to_string(),
                    caption: None,
                    bytes: b"fake-image-bytes".to_vec(),
                },
            )
            .await
            .expect("attach sample media");

        let app = router(context);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/{}", media.relative_file_path))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("media route response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read body");
        assert_eq!(&body[..], b"fake-image-bytes");
    }

    #[tokio::test]
    async fn status_page_renders_checks() {
        let sandbox = WebSandbox::new();
        let context = sandbox.context().await;

        context
            .service
            .create_location(NewLocation {
                name: "Entry Closet".to_string(),
                location_type: "Closet".to_string(),
                parent_id: None,
                notes: None,
            })
            .await
            .expect("create sample location");
        context
            .service
            .create_trip(NewTrip {
                name: "Milan Work Trip".to_string(),
                destination: Some("Milan".to_string()),
                start_date: None,
                end_date: None,
                trip_type: Some("work".to_string()),
                luggage_type: None,
                notes: None,
            })
            .await
            .expect("create sample trip");

        let app = router(context);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("status route response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read body");
        let html = String::from_utf8(body.to_vec()).expect("valid utf-8 html");

        assert!(html.contains("System Status"));
        assert!(html.contains("service layer can read wardrobe counts"));
        assert!(html.contains("PASS"));
    }

    struct WebSandbox {
        root: PathBuf,
        data_dir: PathBuf,
    }

    impl WebSandbox {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);

            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let root = env::temp_dir().join(format!(
                "mywardrobehelper-web-test-{}-{}",
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

    impl Drop for WebSandbox {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
