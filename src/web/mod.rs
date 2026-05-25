use askama::Template;
use std::collections::{HashMap, HashSet};

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
    HealthSnapshot, Item, ItemMedia, Location, Movement, NewItem, NewItemMediaInput, NewLocation,
    NewTrip, NewTripItem, Trip, UpdateItemInput, UpdateTripInput, UpdateTripItemInput,
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
struct TripRow {
    id: String,
    name: String,
    destination: String,
    date_range: String,
}

#[derive(Debug, Clone)]
struct LocationOption {
    id: String,
    label: String,
}

#[derive(Debug, Clone)]
struct LocationRow {
    id: String,
    path: String,
    location_type: String,
    parent: String,
}

#[derive(Debug, Clone)]
struct MovementView {
    moved_at: String,
    from_location: String,
    to_location: String,
    reason: String,
    notes: String,
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
    move_url: String,
}

#[derive(Debug, Clone)]
struct TripDetailView {
    id: String,
    name: String,
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
    notes: String,
}

#[derive(Debug, Clone)]
struct TripFormView {
    name: String,
    destination: String,
    start_date: String,
    end_date: String,
    trip_type: String,
    luggage_type: String,
    notes: String,
}

#[derive(Debug, Clone)]
struct TripItemView {
    id: String,
    item_name: String,
    planned_day: String,
    status: String,
    notes: String,
    update_url: String,
    delete_url: String,
}

#[derive(Debug, Clone)]
struct ItemOption {
    id: String,
    label: String,
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

#[derive(Debug, Deserialize)]
struct LocationFormData {
    name: String,
    location_type: String,
    parent_id: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MoveItemFormData {
    to_location_id: Option<String>,
    reason: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TripFormData {
    name: String,
    destination: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    trip_type: Option<String>,
    luggage_type: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TripItemFormData {
    item_id: String,
    planned_day: Option<String>,
    status: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TripItemUpdateFormData {
    planned_day: Option<String>,
    status: Option<String>,
    notes: Option<String>,
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    page_title: &'static str,
    nav_home_active: bool,
    nav_items_active: bool,
    nav_locations_active: bool,
    nav_trips_active: bool,
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
    nav_locations_active: bool,
    nav_trips_active: bool,
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
    nav_locations_active: bool,
    nav_trips_active: bool,
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
    nav_locations_active: bool,
    nav_trips_active: bool,
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
    nav_locations_active: bool,
    nav_trips_active: bool,
    nav_status_active: bool,
    data_dir: String,
    item: ItemDetailView,
    fields: Vec<ItemField>,
    current_location: String,
    has_movements: bool,
    movements: Vec<MovementView>,
    location_options: Vec<LocationOption>,
    has_media: bool,
    media: Vec<MediaView>,
}

#[derive(Template)]
#[template(path = "locations.html")]
struct LocationsTemplate {
    page_title: &'static str,
    nav_home_active: bool,
    nav_items_active: bool,
    nav_locations_active: bool,
    nav_trips_active: bool,
    nav_status_active: bool,
    data_dir: String,
    has_locations: bool,
    locations: Vec<LocationRow>,
    parent_options: Vec<LocationOption>,
}

#[derive(Template)]
#[template(path = "trips.html")]
struct TripsTemplate {
    page_title: &'static str,
    nav_home_active: bool,
    nav_items_active: bool,
    nav_locations_active: bool,
    nav_trips_active: bool,
    nav_status_active: bool,
    data_dir: String,
    has_trips: bool,
    trips: Vec<TripRow>,
}

#[derive(Template)]
#[template(path = "trip_detail.html")]
struct TripDetailTemplate {
    page_title: String,
    nav_home_active: bool,
    nav_items_active: bool,
    nav_locations_active: bool,
    nav_trips_active: bool,
    nav_status_active: bool,
    data_dir: String,
    trip: TripDetailView,
    trip_form: TripFormView,
    item_options: Vec<ItemOption>,
    has_trip_items: bool,
    trip_items: Vec<TripItemView>,
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
        .route("/items/{id}/move", post(item_move_handler))
        .route(
            "/items/{id}/edit",
            get(item_edit_handler).post(item_update_handler),
        )
        .route("/items/{id}/media", post(item_media_upload_handler))
        .route(
            "/locations",
            get(locations_handler).post(location_create_handler),
        )
        .route("/trips", get(trips_handler).post(trip_create_handler))
        .route("/trips/{id}", get(trip_detail_handler))
        .route("/trips/{id}/edit", post(trip_update_handler))
        .route("/trips/{id}/items", post(trip_item_add_handler))
        .route(
            "/trips/{id}/items/{trip_item_id}/edit",
            post(trip_item_update_handler),
        )
        .route(
            "/trips/{id}/items/{trip_item_id}/delete",
            post(trip_item_delete_handler),
        )
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
        nav_locations_active: false,
        nav_trips_active: false,
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
        nav_locations_active: false,
        nav_trips_active: false,
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
        nav_locations_active: false,
        nav_trips_active: false,
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
    let locations = state
        .context
        .service
        .list_locations()
        .await
        .map_err(internal_error_status)?;
    let movements = state
        .context
        .service
        .get_item_movements(&id)
        .await
        .map_err(internal_error_status)?;
    let location_paths = build_location_path_map(&locations);

    let template = ItemDetailTemplate {
        page_title: format!("Item · {}", item.name),
        nav_home_active: false,
        nav_items_active: true,
        nav_locations_active: false,
        nav_trips_active: false,
        nav_status_active: false,
        data_dir: state.context.layout.root.display().to_string(),
        item: ItemDetailView {
            id: item.id.clone(),
            name: item.name.clone(),
            edit_url: format!("/items/{}/edit", item.id),
            upload_url: format!("/items/{}/media", item.id),
            move_url: format!("/items/{}/move", item.id),
        },
        fields: item_fields(&item),
        current_location: item
            .current_location_id
            .as_deref()
            .and_then(|location_id| location_paths.get(location_id).cloned())
            .unwrap_or_else(|| "Not assigned".to_string()),
        has_movements: !movements.is_empty(),
        movements: movements
            .into_iter()
            .map(|movement| movement_view(movement, &location_paths))
            .collect(),
        location_options: location_options(&locations),
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
        nav_locations_active: false,
        nav_trips_active: false,
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
    let existing = state
        .context
        .service
        .get_item(&id)
        .await
        .map_err(internal_error_status)?
        .ok_or(StatusCode::NOT_FOUND)?;

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
                current_location_id: existing.current_location_id,
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

async fn item_move_handler(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(form): Form<MoveItemFormData>,
) -> Result<Redirect, StatusCode> {
    state
        .context
        .service
        .move_item(
            &id,
            crate::domain::MoveItemInput {
                to_location_id: form.to_location_id,
                reason: form.reason,
                notes: form.notes,
            },
        )
        .await
        .map_err(internal_error_status)?;

    Ok(Redirect::to(&format!("/items/{id}")))
}

async fn locations_handler(State(state): State<WebState>) -> Result<Html<String>, StatusCode> {
    let locations = state
        .context
        .service
        .list_locations()
        .await
        .map_err(internal_error_status)?;
    let location_paths = build_location_path_map(&locations);

    let template = LocationsTemplate {
        page_title: "Locations",
        nav_home_active: false,
        nav_items_active: false,
        nav_locations_active: true,
        nav_trips_active: false,
        nav_status_active: false,
        data_dir: state.context.layout.root.display().to_string(),
        has_locations: !locations.is_empty(),
        locations: locations
            .iter()
            .map(|location| LocationRow {
                id: location.id.clone(),
                path: location_paths
                    .get(&location.id)
                    .cloned()
                    .unwrap_or_else(|| location.name.clone()),
                location_type: location.location_type.clone(),
                parent: location
                    .parent_id
                    .as_deref()
                    .and_then(|parent_id| location_paths.get(parent_id).cloned())
                    .unwrap_or_else(|| "Top-level".to_string()),
            })
            .collect(),
        parent_options: location_options(&locations),
    };

    render_template(&template)
}

async fn location_create_handler(
    State(state): State<WebState>,
    Form(form): Form<LocationFormData>,
) -> Result<Redirect, StatusCode> {
    state
        .context
        .service
        .create_location(NewLocation {
            name: form.name,
            location_type: form.location_type,
            parent_id: form.parent_id,
            notes: form.notes,
        })
        .await
        .map_err(internal_error_status)?;

    Ok(Redirect::to("/locations"))
}

async fn trips_handler(State(state): State<WebState>) -> Result<Html<String>, StatusCode> {
    let trips = state
        .context
        .service
        .list_trips()
        .await
        .map_err(internal_error_status)?;

    let template = TripsTemplate {
        page_title: "Trips",
        nav_home_active: false,
        nav_items_active: false,
        nav_locations_active: false,
        nav_trips_active: true,
        nav_status_active: false,
        data_dir: state.context.layout.root.display().to_string(),
        has_trips: !trips.is_empty(),
        trips: trips
            .into_iter()
            .map(|trip| TripRow {
                id: trip.id.clone(),
                name: trip.name,
                destination: trip
                    .destination
                    .unwrap_or_else(|| "No destination set".to_string()),
                date_range: join_optional_parts([
                    trip.start_date.as_deref(),
                    trip.end_date.as_deref(),
                ]),
            })
            .collect(),
    };

    render_template(&template)
}

async fn trip_create_handler(
    State(state): State<WebState>,
    Form(form): Form<TripFormData>,
) -> Result<Redirect, StatusCode> {
    let trip = state
        .context
        .service
        .create_trip(NewTrip {
            name: form.name,
            destination: form.destination,
            start_date: form.start_date,
            end_date: form.end_date,
            trip_type: form.trip_type,
            luggage_type: form.luggage_type,
            notes: form.notes,
        })
        .await
        .map_err(internal_error_status)?;

    Ok(Redirect::to(&format!("/trips/{}", trip.id)))
}

async fn trip_detail_handler(
    State(state): State<WebState>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let trip = state
        .context
        .service
        .get_trip(&id)
        .await
        .map_err(internal_error_status)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let items = state
        .context
        .service
        .list_items()
        .await
        .map_err(internal_error_status)?;
    let trip_items = state
        .context
        .service
        .list_trip_items(&id)
        .await
        .map_err(internal_error_status)?;

    let template = TripDetailTemplate {
        page_title: format!("Trip · {}", trip.name),
        nav_home_active: false,
        nav_items_active: false,
        nav_locations_active: false,
        nav_trips_active: true,
        nav_status_active: false,
        data_dir: state.context.layout.root.display().to_string(),
        trip: TripDetailView {
            id: trip.id.clone(),
            name: trip.name.clone(),
        },
        trip_form: trip_form_view(&trip),
        item_options: items
            .into_iter()
            .map(|item| ItemOption {
                id: item.id.clone(),
                label: format!("{} ({})", item.name, item.id),
            })
            .collect(),
        has_trip_items: !trip_items.is_empty(),
        trip_items: trip_items
            .into_iter()
            .map(|trip_item| TripItemView {
                id: trip_item.id.clone(),
                item_name: trip_item
                    .item_name
                    .unwrap_or_else(|| trip_item.item_id.clone()),
                planned_day: trip_item.planned_day.unwrap_or_default(),
                status: trip_item.status.unwrap_or_default(),
                notes: trip_item.notes.unwrap_or_default(),
                update_url: format!("/trips/{}/items/{}/edit", id, trip_item.id),
                delete_url: format!("/trips/{}/items/{}/delete", id, trip_item.id),
            })
            .collect(),
    };

    render_template(&template)
}

async fn trip_update_handler(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(form): Form<TripFormData>,
) -> Result<Redirect, StatusCode> {
    state
        .context
        .service
        .update_trip(
            &id,
            UpdateTripInput {
                name: form.name,
                destination: form.destination,
                start_date: form.start_date,
                end_date: form.end_date,
                trip_type: form.trip_type,
                luggage_type: form.luggage_type,
                notes: form.notes,
            },
        )
        .await
        .map_err(internal_error_status)?;

    Ok(Redirect::to(&format!("/trips/{id}")))
}

async fn trip_item_add_handler(
    State(state): State<WebState>,
    Path(id): Path<String>,
    Form(form): Form<TripItemFormData>,
) -> Result<Redirect, StatusCode> {
    state
        .context
        .service
        .add_trip_item(
            &id,
            NewTripItem {
                item_id: form.item_id,
                planned_day: form.planned_day,
                status: form.status,
                notes: form.notes,
            },
        )
        .await
        .map_err(internal_error_status)?;

    Ok(Redirect::to(&format!("/trips/{id}")))
}

async fn trip_item_update_handler(
    State(state): State<WebState>,
    Path((id, trip_item_id)): Path<(String, String)>,
    Form(form): Form<TripItemUpdateFormData>,
) -> Result<Redirect, StatusCode> {
    state
        .context
        .service
        .update_trip_item(
            &id,
            &trip_item_id,
            UpdateTripItemInput {
                planned_day: form.planned_day,
                status: form.status,
                notes: form.notes,
            },
        )
        .await
        .map_err(internal_error_status)?;

    Ok(Redirect::to(&format!("/trips/{id}")))
}

async fn trip_item_delete_handler(
    State(state): State<WebState>,
    Path((id, trip_item_id)): Path<(String, String)>,
) -> Result<Redirect, StatusCode> {
    state
        .context
        .service
        .remove_trip_item(&id, &trip_item_id)
        .await
        .map_err(internal_error_status)?;

    Ok(Redirect::to(&format!("/trips/{id}")))
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
        nav_locations_active: false,
        nav_trips_active: false,
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
        notes: item.notes.clone().unwrap_or_default(),
    }
}

fn location_options(locations: &[Location]) -> Vec<LocationOption> {
    let paths = build_location_path_map(locations);
    locations
        .iter()
        .map(|location| LocationOption {
            id: location.id.clone(),
            label: paths
                .get(&location.id)
                .cloned()
                .unwrap_or_else(|| location.name.clone()),
        })
        .collect()
}

fn build_location_path_map(locations: &[Location]) -> HashMap<String, String> {
    let by_id: HashMap<&str, &Location> = locations
        .iter()
        .map(|location| (location.id.as_str(), location))
        .collect();
    let mut cache = HashMap::new();

    for location in locations {
        let path = build_location_path(
            location.id.as_str(),
            &by_id,
            &mut cache,
            &mut HashSet::new(),
        );
        cache.insert(location.id.clone(), path);
    }

    cache
}

fn build_location_path(
    location_id: &str,
    by_id: &HashMap<&str, &Location>,
    cache: &mut HashMap<String, String>,
    visiting: &mut HashSet<String>,
) -> String {
    if let Some(existing) = cache.get(location_id) {
        return existing.clone();
    }

    if !visiting.insert(location_id.to_string()) {
        return location_id.to_string();
    }

    let Some(location) = by_id.get(location_id).copied() else {
        return location_id.to_string();
    };

    let path = match location.parent_id.as_deref() {
        Some(parent_id) => {
            let parent_path = build_location_path(parent_id, by_id, cache, visiting);
            format!("{parent_path} > {}", location.name)
        }
        None => location.name.clone(),
    };

    visiting.remove(location_id);
    cache.insert(location_id.to_string(), path.clone());
    path
}

fn movement_view(movement: Movement, location_paths: &HashMap<String, String>) -> MovementView {
    MovementView {
        moved_at: movement.moved_at,
        from_location: movement
            .from_location_id
            .as_deref()
            .and_then(|id| location_paths.get(id).cloned())
            .unwrap_or_else(|| "Unassigned".to_string()),
        to_location: movement
            .to_location_id
            .as_deref()
            .and_then(|id| location_paths.get(id).cloned())
            .unwrap_or_else(|| "Unassigned".to_string()),
        reason: movement
            .reason
            .unwrap_or_else(|| "No reason recorded".to_string()),
        notes: movement.notes.unwrap_or_default(),
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

fn trip_form_view(trip: &Trip) -> TripFormView {
    TripFormView {
        name: trip.name.clone(),
        destination: trip.destination.clone().unwrap_or_default(),
        start_date: trip.start_date.clone().unwrap_or_default(),
        end_date: trip.end_date.clone().unwrap_or_default(),
        trip_type: trip.trip_type.clone().unwrap_or_default(),
        luggage_type: trip.luggage_type.clone().unwrap_or_default(),
        notes: trip.notes.clone().unwrap_or_default(),
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
    use crate::domain::{NewItem, NewItemMediaInput, NewLocation, NewTrip, NewTripItem};

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
    async fn locations_page_renders_nested_paths() {
        let sandbox = WebSandbox::new();
        let context = sandbox.context().await;
        let root = context
            .service
            .create_location(NewLocation {
                name: "Treviso House".to_string(),
                location_type: "House".to_string(),
                parent_id: None,
                notes: None,
            })
            .await
            .expect("create root location");
        context
            .service
            .create_location(NewLocation {
                name: "Bedroom Closet".to_string(),
                location_type: "Closet".to_string(),
                parent_id: Some(root.id),
                notes: None,
            })
            .await
            .expect("create nested location");

        let app = router(context);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/locations")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("locations route response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read body");
        let html = String::from_utf8(body.to_vec()).expect("valid utf-8 html");

        assert!(html.contains("Treviso House"));
        assert!(html.contains("Bedroom Closet"));
        assert!(html.contains("Top-level"));
    }

    #[tokio::test]
    async fn item_detail_page_renders_movement_history() {
        let sandbox = WebSandbox::new();
        let context = sandbox.context().await;
        let location = context
            .service
            .create_location(NewLocation {
                name: "Suitcase".to_string(),
                location_type: "Luggage".to_string(),
                parent_id: None,
                notes: None,
            })
            .await
            .expect("create location");
        let item = context
            .service
            .create_item(NewItem {
                name: "Travel Tee".to_string(),
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
            .expect("create item");
        context
            .service
            .move_item(
                &item.id,
                crate::domain::MoveItemInput {
                    to_location_id: Some(location.id),
                    reason: Some("packing".to_string()),
                    notes: None,
                },
            )
            .await
            .expect("move item");

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

        assert!(html.contains("Movement History"));
        assert!(html.contains("packing"));
        assert!(html.contains("Suitcase"));
    }

    #[tokio::test]
    async fn trips_page_renders_created_trip() {
        let sandbox = WebSandbox::new();
        let context = sandbox.context().await;
        context
            .service
            .create_trip(NewTrip {
                name: "Berlin Weekend".to_string(),
                destination: Some("Berlin".to_string()),
                start_date: Some("2026-06-01".to_string()),
                end_date: Some("2026-06-03".to_string()),
                trip_type: Some("leisure".to_string()),
                luggage_type: Some("carry-on".to_string()),
                notes: None,
            })
            .await
            .expect("create trip");

        let app = router(context);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/trips")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("trips route response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read body");
        let html = String::from_utf8(body.to_vec()).expect("valid utf-8 html");

        assert!(html.contains("Berlin Weekend"));
        assert!(html.contains("carry-on") || html.contains("Berlin"));
    }

    #[tokio::test]
    async fn trip_detail_page_renders_packing_list() {
        let sandbox = WebSandbox::new();
        let context = sandbox.context().await;
        let item = context
            .service
            .create_item(NewItem {
                name: "Merino Tee".to_string(),
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
            .expect("create item");
        let trip = context
            .service
            .create_trip(NewTrip {
                name: "Rome Weekend".to_string(),
                destination: None,
                start_date: None,
                end_date: None,
                trip_type: None,
                luggage_type: None,
                notes: None,
            })
            .await
            .expect("create trip");
        context
            .service
            .add_trip_item(
                &trip.id,
                NewTripItem {
                    item_id: item.id,
                    planned_day: Some("day-1".to_string()),
                    status: Some("planned".to_string()),
                    notes: Some("Pack in outer pocket".to_string()),
                },
            )
            .await
            .expect("add trip item");

        let app = router(context);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/trips/{}", trip.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("trip detail response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read body");
        let html = String::from_utf8(body.to_vec()).expect("valid utf-8 html");

        assert!(html.contains("Packing List"));
        assert!(html.contains("Merino Tee"));
        assert!(html.contains("Pack in outer pocket"));
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
