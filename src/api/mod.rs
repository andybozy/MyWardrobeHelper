use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::AppContext;
use crate::domain::{
    Item, ItemMedia, Location, MoveItemInput, Movement, NewItem, NewItemMediaInput, NewLocation,
    NewPhysicalTag, NewTrip, NewTripItem, PhysicalTag, ResolvePhysicalTagInput,
    ResolvedPhysicalTag, Trip, TripItem, UpdateItemInput, UpdateTripInput, UpdateTripItemInput,
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
struct TripsListResponse {
    trips: Vec<TripResponse>,
}

#[derive(Debug, Serialize)]
struct PhysicalTagsListResponse {
    tags: Vec<PhysicalTagResponse>,
}

#[derive(Debug, Serialize)]
struct ItemMediaListResponse {
    media: Vec<ItemMediaResponse>,
}

#[derive(Debug, Serialize)]
struct TripItemsListResponse {
    trip_items: Vec<TripItemResponse>,
}

#[derive(Debug, Serialize)]
struct ResolvedPhysicalTagResponse {
    tag: PhysicalTagResponse,
    entity_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct MovementsListResponse {
    movements: Vec<MovementResponse>,
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
struct MovementResponse {
    id: String,
    item_id: String,
    from_location_id: Option<String>,
    to_location_id: Option<String>,
    reason: Option<String>,
    notes: Option<String>,
    moved_at: String,
}

#[derive(Debug, Serialize)]
struct TripResponse {
    id: String,
    name: String,
    destination: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    trip_type: Option<String>,
    luggage_type: Option<String>,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct TripItemResponse {
    id: String,
    trip_id: String,
    item_id: String,
    item_name: Option<String>,
    planned_day: Option<String>,
    status: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Serialize)]
struct PhysicalTagResponse {
    id: String,
    tag_type: String,
    external_identifier: String,
    label: Option<String>,
    bound_entity_type: String,
    bound_entity_id: String,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
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
struct CreateTripRequest {
    name: String,
    destination: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    trip_type: Option<String>,
    luggage_type: Option<String>,
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

#[derive(Debug, Deserialize)]
struct MoveItemRequest {
    to_location_id: Option<String>,
    reason: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PatchTripRequest {
    name: Option<String>,
    destination: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    trip_type: Option<String>,
    luggage_type: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateTripItemRequest {
    item_id: String,
    planned_day: Option<String>,
    status: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreatePhysicalTagRequest {
    tag_type: String,
    external_identifier: String,
    label: Option<String>,
    bound_entity_type: String,
    bound_entity_id: String,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PatchTripItemRequest {
    planned_day: Option<String>,
    status: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResolvePhysicalTagRequest {
    tag_type: String,
    external_identifier: String,
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
        .route("/items/{id}/move", post(move_item_handler))
        .route("/items/{id}/movements", get(list_item_movements_handler))
        .route(
            "/items/{id}/media",
            get(list_item_media_handler).post(upload_item_media_handler),
        )
        .route(
            "/locations",
            get(list_locations_handler).post(create_location_handler),
        )
        .route("/locations/{id}", get(get_location_handler))
        .route("/trips", get(list_trips_handler).post(create_trip_handler))
        .route(
            "/trips/{id}",
            get(get_trip_handler).patch(update_trip_handler),
        )
        .route(
            "/trips/{id}/items",
            get(list_trip_items_handler).post(add_trip_item_handler),
        )
        .route(
            "/trips/{id}/items/{trip_item_id}",
            axum::routing::patch(update_trip_item_handler).delete(delete_trip_item_handler),
        )
        .route("/tags", get(list_tags_handler).post(create_tag_handler))
        .route("/tags/resolve", post(resolve_tag_handler))
        .route("/tags/{id}", get(get_tag_handler))
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
    if request.current_location_id.is_some() {
        return Err(ApiError::bad_request(
            "USE_MOVE_ENDPOINT",
            "Use POST /api/v1/items/:id/move to change an item's location.".to_string(),
            Some(json!({ "item_id": id })),
        ));
    }

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

async fn move_item_handler(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(request): Json<MoveItemRequest>,
) -> Result<Json<MovementResponse>, ApiError> {
    let result = state
        .context
        .service
        .move_item(
            &id,
            MoveItemInput {
                to_location_id: request.to_location_id,
                reason: request.reason,
                notes: request.notes,
            },
        )
        .await
        .map_err(item_related_error(&id))?;

    Ok(Json(MovementResponse::from(result.movement)))
}

async fn list_item_movements_handler(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<MovementsListResponse>, ApiError> {
    let movements = state
        .context
        .service
        .get_item_movements(&id)
        .await
        .map_err(item_related_error(&id))?;

    Ok(Json(MovementsListResponse {
        movements: movements.into_iter().map(MovementResponse::from).collect(),
    }))
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

async fn list_trips_handler(
    State(state): State<ApiState>,
) -> Result<Json<TripsListResponse>, ApiError> {
    let trips = state
        .context
        .service
        .list_trips()
        .await
        .map_err(ApiError::from)?;

    Ok(Json(TripsListResponse {
        trips: trips.into_iter().map(TripResponse::from).collect(),
    }))
}

async fn create_trip_handler(
    State(state): State<ApiState>,
    Json(request): Json<CreateTripRequest>,
) -> Result<(StatusCode, Json<TripResponse>), ApiError> {
    let trip = state
        .context
        .service
        .create_trip(NewTrip {
            name: request.name,
            destination: request.destination,
            start_date: request.start_date,
            end_date: request.end_date,
            trip_type: request.trip_type,
            luggage_type: request.luggage_type,
            notes: request.notes,
        })
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(TripResponse::from(trip))))
}

async fn get_trip_handler(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<TripResponse>, ApiError> {
    let trip = state
        .context
        .service
        .get_trip(&id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| {
            ApiError::not_found(
                "TRIP_NOT_FOUND",
                "Trip not found",
                Some(json!({ "trip_id": id })),
            )
        })?;

    Ok(Json(TripResponse::from(trip)))
}

async fn update_trip_handler(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(request): Json<PatchTripRequest>,
) -> Result<Json<TripResponse>, ApiError> {
    let existing = state
        .context
        .service
        .get_trip(&id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| {
            ApiError::not_found(
                "TRIP_NOT_FOUND",
                "Trip not found",
                Some(json!({ "trip_id": id.clone() })),
            )
        })?;

    let updated = state
        .context
        .service
        .update_trip(
            &id,
            UpdateTripInput {
                name: request.name.unwrap_or(existing.name),
                destination: request.destination.or(existing.destination),
                start_date: request.start_date.or(existing.start_date),
                end_date: request.end_date.or(existing.end_date),
                trip_type: request.trip_type.or(existing.trip_type),
                luggage_type: request.luggage_type.or(existing.luggage_type),
                notes: request.notes.or(existing.notes),
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Json(TripResponse::from(updated)))
}

async fn list_trip_items_handler(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<TripItemsListResponse>, ApiError> {
    let trip_items = state
        .context
        .service
        .list_trip_items(&id)
        .await
        .map_err(trip_related_error(&id))?;

    Ok(Json(TripItemsListResponse {
        trip_items: trip_items.into_iter().map(TripItemResponse::from).collect(),
    }))
}

async fn add_trip_item_handler(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(request): Json<CreateTripItemRequest>,
) -> Result<(StatusCode, Json<TripItemResponse>), ApiError> {
    let trip_item = state
        .context
        .service
        .add_trip_item(
            &id,
            NewTripItem {
                item_id: request.item_id,
                planned_day: request.planned_day,
                status: request.status,
                notes: request.notes,
            },
        )
        .await
        .map_err(trip_related_error(&id))?;

    Ok((StatusCode::CREATED, Json(TripItemResponse::from(trip_item))))
}

async fn update_trip_item_handler(
    State(state): State<ApiState>,
    Path((id, trip_item_id)): Path<(String, String)>,
    Json(request): Json<PatchTripItemRequest>,
) -> Result<Json<TripItemResponse>, ApiError> {
    let trip_item = state
        .context
        .service
        .update_trip_item(
            &id,
            &trip_item_id,
            UpdateTripItemInput {
                planned_day: request.planned_day,
                status: request.status,
                notes: request.notes,
            },
        )
        .await
        .map_err(trip_related_error(&id))?;

    Ok(Json(TripItemResponse::from(trip_item)))
}

async fn delete_trip_item_handler(
    State(state): State<ApiState>,
    Path((id, trip_item_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    state
        .context
        .service
        .remove_trip_item(&id, &trip_item_id)
        .await
        .map_err(trip_related_error(&id))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn list_tags_handler(
    State(state): State<ApiState>,
) -> Result<Json<PhysicalTagsListResponse>, ApiError> {
    let tags = state
        .context
        .service
        .list_physical_tags()
        .await
        .map_err(ApiError::from)?;

    Ok(Json(PhysicalTagsListResponse {
        tags: tags.into_iter().map(PhysicalTagResponse::from).collect(),
    }))
}

async fn create_tag_handler(
    State(state): State<ApiState>,
    Json(request): Json<CreatePhysicalTagRequest>,
) -> Result<(StatusCode, Json<PhysicalTagResponse>), ApiError> {
    let tag = state
        .context
        .service
        .register_physical_tag(NewPhysicalTag {
            tag_type: request.tag_type,
            external_identifier: request.external_identifier,
            label: request.label,
            bound_entity_type: request.bound_entity_type,
            bound_entity_id: request.bound_entity_id,
            notes: request.notes,
        })
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(PhysicalTagResponse::from(tag))))
}

async fn get_tag_handler(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<PhysicalTagResponse>, ApiError> {
    let tag = state
        .context
        .service
        .get_physical_tag(&id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| {
            ApiError::not_found(
                "TAG_NOT_FOUND",
                "Physical tag not found",
                Some(json!({ "tag_id": id })),
            )
        })?;

    Ok(Json(PhysicalTagResponse::from(tag)))
}

async fn resolve_tag_handler(
    State(state): State<ApiState>,
    Json(request): Json<ResolvePhysicalTagRequest>,
) -> Result<Json<ResolvedPhysicalTagResponse>, ApiError> {
    let tag_type = request.tag_type.clone();
    let external_identifier = request.external_identifier.clone();
    let resolved = state
        .context
        .service
        .resolve_physical_tag(ResolvePhysicalTagInput {
            tag_type,
            external_identifier,
        })
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| {
            ApiError::not_found(
                "TAG_NOT_FOUND",
                "Physical tag not found",
                Some(json!({
                    "tag_type": request.tag_type,
                    "external_identifier": request.external_identifier
                })),
            )
        })?;

    Ok(Json(ResolvedPhysicalTagResponse::from(resolved)))
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

impl From<Movement> for MovementResponse {
    fn from(value: Movement) -> Self {
        Self {
            id: value.id,
            item_id: value.item_id,
            from_location_id: value.from_location_id,
            to_location_id: value.to_location_id,
            reason: value.reason,
            notes: value.notes,
            moved_at: value.moved_at,
        }
    }
}

impl From<Trip> for TripResponse {
    fn from(value: Trip) -> Self {
        Self {
            id: value.id,
            name: value.name,
            destination: value.destination,
            start_date: value.start_date,
            end_date: value.end_date,
            trip_type: value.trip_type,
            luggage_type: value.luggage_type,
            notes: value.notes,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<TripItem> for TripItemResponse {
    fn from(value: TripItem) -> Self {
        Self {
            id: value.id,
            trip_id: value.trip_id,
            item_id: value.item_id,
            item_name: value.item_name,
            planned_day: value.planned_day,
            status: value.status,
            notes: value.notes,
        }
    }
}

impl From<PhysicalTag> for PhysicalTagResponse {
    fn from(value: PhysicalTag) -> Self {
        Self {
            id: value.id,
            tag_type: value.tag_type,
            external_identifier: value.external_identifier,
            label: value.label,
            bound_entity_type: value.bound_entity_type,
            bound_entity_id: value.bound_entity_id,
            notes: value.notes,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<ResolvedPhysicalTag> for ResolvedPhysicalTagResponse {
    fn from(value: ResolvedPhysicalTag) -> Self {
        Self {
            tag: PhysicalTagResponse::from(value.tag),
            entity_name: value.entity_name,
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

fn trip_related_error(trip_id: &str) -> impl Fn(AppError) -> ApiError + '_ {
    move |error| match error {
        AppError::InvalidArgument(message)
            if message.contains("trip `") && message.contains("does not exist") =>
        {
            ApiError::not_found(
                "TRIP_NOT_FOUND",
                "Trip not found",
                Some(json!({ "trip_id": trip_id })),
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
    async fn item_move_and_movement_routes_work() {
        let sandbox = ApiSandbox::new();
        let app = router(sandbox.context().await);

        let created_item = app
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
        let item_body = to_json(created_item.into_body()).await;
        let item_id = item_body["id"].as_str().unwrap().to_string();

        let created_location = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/locations")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({ "name": "Suitcase", "location_type": "Luggage" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("create location response");
        let location_body = to_json(created_location.into_body()).await;
        let location_id = location_body["id"].as_str().unwrap().to_string();

        let moved = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/move"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({ "to_location_id": location_id, "reason": "packing" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("move item response");

        assert_eq!(moved.status(), StatusCode::OK);
        let moved_body = to_json(moved.into_body()).await;
        assert_eq!(moved_body["reason"], "packing");

        let movements = app
            .oneshot(
                Request::builder()
                    .uri(format!("/items/{item_id}/movements"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("list movements response");

        assert_eq!(movements.status(), StatusCode::OK);
        let movements_body = to_json(movements.into_body()).await;
        assert_eq!(movements_body["movements"][0]["reason"], "packing");
    }

    #[tokio::test]
    async fn item_patch_rejects_direct_location_change() {
        let sandbox = ApiSandbox::new();
        let app = router(sandbox.context().await);

        let created_item = app
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
        let item_body = to_json(created_item.into_body()).await;
        let item_id = item_body["id"].as_str().unwrap().to_string();

        let rejected = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/items/{item_id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({ "current_location_id": "location-123" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("patch response");

        assert_eq!(rejected.status(), StatusCode::BAD_REQUEST);
        let body = to_json(rejected.into_body()).await;
        assert_eq!(body["error"]["code"], "USE_MOVE_ENDPOINT");
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
    async fn trip_routes_support_create_get_patch_and_trip_items() {
        let sandbox = ApiSandbox::new();
        let app = router(sandbox.context().await);

        let created_item = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/items")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({ "name": "Merino Tee" }).to_string()))
                    .unwrap(),
            )
            .await
            .expect("create item response");
        let item_body = to_json(created_item.into_body()).await;
        let item_id = item_body["id"].as_str().unwrap().to_string();

        let created_trip = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/trips")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({ "name": "Rome Weekend", "destination": "Rome" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("create trip response");
        assert_eq!(created_trip.status(), StatusCode::CREATED);
        let trip_body = to_json(created_trip.into_body()).await;
        let trip_id = trip_body["id"].as_str().unwrap().to_string();

        let patched_trip = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/trips/{trip_id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({ "luggage_type": "carry-on", "notes": "Two nights" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("patch trip response");
        assert_eq!(patched_trip.status(), StatusCode::OK);
        let patched_trip_body = to_json(patched_trip.into_body()).await;
        assert_eq!(patched_trip_body["luggage_type"], "carry-on");

        let added_trip_item = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/trips/{trip_id}/items"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "item_id": item_id,
                            "planned_day": "day-1",
                            "status": "planned"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("add trip item response");
        assert_eq!(added_trip_item.status(), StatusCode::CREATED);
        let trip_item_body = to_json(added_trip_item.into_body()).await;
        let trip_item_id = trip_item_body["id"].as_str().unwrap().to_string();

        let updated_trip_item = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/trips/{trip_id}/items/{trip_item_id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({ "status": "packed", "notes": "Packed in top section" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("patch trip item response");
        assert_eq!(updated_trip_item.status(), StatusCode::OK);
        let updated_trip_item_body = to_json(updated_trip_item.into_body()).await;
        assert_eq!(updated_trip_item_body["status"], "packed");

        let trip_items = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/trips/{trip_id}/items"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("list trip items response");
        assert_eq!(trip_items.status(), StatusCode::OK);
        let trip_items_body = to_json(trip_items.into_body()).await;
        assert_eq!(
            trip_items_body["trip_items"][0]["notes"],
            "Packed in top section"
        );

        let deleted = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/trips/{trip_id}/items/{trip_item_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("delete trip item response");
        assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn physical_tag_routes_support_create_get_list_and_resolve() {
        let sandbox = ApiSandbox::new();
        let app = router(sandbox.context().await);

        let created_item = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/items")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({ "name": "Corduroy Overshirt" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("create item response");
        let item_body = to_json(created_item.into_body()).await;
        let item_id = item_body["id"].as_str().unwrap().to_string();

        let created_tag = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tags")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "tag_type": "nfc",
                            "external_identifier": "04-A2-88-FF",
                            "label": "Overshirt NFC",
                            "bound_entity_type": "item",
                            "bound_entity_id": item_id
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("create tag response");
        assert_eq!(created_tag.status(), StatusCode::CREATED);
        let tag_body = to_json(created_tag.into_body()).await;
        let tag_id = tag_body["id"].as_str().unwrap().to_string();

        let listed_tags = app
            .clone()
            .oneshot(Request::builder().uri("/tags").body(Body::empty()).unwrap())
            .await
            .expect("list tags response");
        assert_eq!(listed_tags.status(), StatusCode::OK);
        let listed_tags_body = to_json(listed_tags.into_body()).await;
        assert_eq!(
            listed_tags_body["tags"][0]["external_identifier"],
            "04-A2-88-FF"
        );

        let fetched_tag = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/tags/{tag_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("get tag response");
        assert_eq!(fetched_tag.status(), StatusCode::OK);

        let resolved_tag = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tags/resolve")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({
                            "tag_type": "nfc",
                            "external_identifier": "04-A2-88-FF"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .expect("resolve tag response");
        assert_eq!(resolved_tag.status(), StatusCode::OK);
        let resolved_body = to_json(resolved_tag.into_body()).await;
        assert_eq!(resolved_body["entity_name"], "Corduroy Overshirt");
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
