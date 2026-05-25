use askama::Template;
use axum::Router;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use tokio::net::TcpListener;

use crate::api;
use crate::app::{self, AppContext};
use crate::domain::{HealthSnapshot, Item, Location, Trip};
use crate::error::{AppError, AppResult};

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

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    page_title: &'static str,
    nav_home_active: bool,
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
    nav_status_active: bool,
    data_dir: String,
    bind_url: String,
    local_url: String,
    lan_url: Option<String>,
    database_file: String,
    health: HealthSnapshot,
    checks: Vec<StatusCheckView>,
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
        .route("/status", get(status_handler))
        .route("/assets/app.css", get(stylesheet_handler))
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
    use crate::domain::{NewItem, NewLocation, NewTrip};

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
