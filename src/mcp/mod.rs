use std::io::{self, BufRead, BufReader as StdBufReader, BufWriter as StdBufWriter, Write};
use std::net::{Shutdown, TcpStream as StdTcpStream};
use std::thread;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};
use tokio::io::{
    AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader, BufWriter as AsyncBufWriter,
};
use tokio::net::{TcpListener, TcpStream};

use crate::app::AppContext;
use crate::config::AppConfig;
use crate::domain::{
    ItemFilter, MoveItemInput, NewItem, NewLocation, NewTrip, NewTripItem, UpdateTripInput,
    UpdateTripItemInput,
};
use crate::error::{AppError, AppResult};

const SUPPORTED_PROTOCOL_VERSIONS: &[&str] =
    &["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];
const SERVER_NAME: &str = "mywardrobehelper";
const SERVER_TITLE: &str = "MyWardrobeHelper MCP";
const MCP_TCP_HOST: &str = "127.0.0.1";

#[derive(Debug, Deserialize)]
struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    protocol_version: String,
}

#[derive(Debug, Deserialize)]
struct ToolCallParams {
    name: String,
    arguments: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ItemIdArgs {
    item_id: String,
}

#[derive(Debug, Deserialize, Default)]
struct ListItemsArgs {
    q: Option<String>,
    category: Option<String>,
    brand: Option<String>,
    season: Option<String>,
    current_location_id: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateItemArgs {
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
struct CreateLocationArgs {
    name: String,
    location_type: String,
    parent_id: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MoveItemArgs {
    item_id: String,
    to_location_id: Option<String>,
    reason: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TripIdArgs {
    trip_id: String,
}

#[derive(Debug, Deserialize)]
struct CreateTripArgs {
    name: String,
    destination: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    trip_type: Option<String>,
    luggage_type: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateTripArgs {
    trip_id: String,
    name: Option<String>,
    destination: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    trip_type: Option<String>,
    luggage_type: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AddTripItemArgs {
    trip_id: String,
    item_id: String,
    planned_day: Option<String>,
    status: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateTripItemArgs {
    trip_id: String,
    trip_item_id: String,
    planned_day: Option<String>,
    status: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RemoveTripItemArgs {
    trip_id: String,
    trip_item_id: String,
}

#[derive(Debug, Deserialize)]
struct AnalyzeItemPhotoArgs {
    image_path: String,
}

pub async fn serve(context: AppContext) -> AppResult<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut server = McpServer::new(context);
    let mut writer = StdBufWriter::new(stdout.lock());

    for line in StdBufReader::new(stdin.lock()).lines() {
        let line = line.map_err(|error| AppError::io("read MCP stdin", error))?;
        if line.trim().is_empty() {
            continue;
        }

        let responses = server.handle_line(&line).await;
        for response in responses {
            serde_json::to_writer(&mut writer, &response)
                .map_err(|error| AppError::config(format!("serialize MCP response: {error}")))?;
            writer
                .write_all(b"\n")
                .map_err(|error| AppError::io("write MCP stdout", error))?;
        }
        writer
            .flush()
            .map_err(|error| AppError::io("flush MCP stdout", error))?;
    }

    Ok(())
}

pub async fn serve_tcp(context: AppContext) -> AppResult<()> {
    let port = tcp_port(&context.config)?;
    let bind_target = format!("{MCP_TCP_HOST}:{port}");
    let listener = TcpListener::bind((MCP_TCP_HOST, port))
        .await
        .map_err(|error| AppError::io(format!("bind MCP TCP listener at {bind_target}"), error))?;

    println!("Codex MCP: {}", tcp_url(port));
    println!("Codex bridge: cargo run --release -- mcp connect");

    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            _ = &mut shutdown => return Ok(()),
            accepted = listener.accept() => {
                let (stream, _) = accepted
                    .map_err(|error| AppError::io(format!("accept MCP TCP connection on {bind_target}"), error))?;
                let session_context = context.clone();
                tokio::spawn(async move {
                    if let Err(error) = serve_tcp_session(stream, session_context).await {
                        eprintln!("warning: {error}");
                    }
                });
            }
        }
    }
}

pub async fn connect(config: &AppConfig) -> AppResult<()> {
    let port = tcp_port(config)?;
    let address = format!("{MCP_TCP_HOST}:{port}");
    let mut stream = StdTcpStream::connect(&address).map_err(|error| {
        AppError::io(format!("connect to MCP TCP listener at {address}"), error)
    })?;
    let mut read_stream = stream
        .try_clone()
        .map_err(|error| AppError::io(format!("clone MCP TCP stream for {address}"), error))?;

    let output_thread = thread::spawn(move || -> AppResult<()> {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        io::copy(&mut read_stream, &mut stdout)
            .map_err(|error| AppError::io("forward MCP responses to stdout", error))?;
        stdout
            .flush()
            .map_err(|error| AppError::io("flush stdout", error))?;
        Ok(())
    });

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    io::copy(&mut stdin, &mut stream)
        .map_err(|error| AppError::io("forward MCP requests from stdin", error))?;
    stream.shutdown(Shutdown::Write).map_err(|error| {
        AppError::io(format!("shutdown MCP TCP write half for {address}"), error)
    })?;

    match output_thread.join() {
        Ok(result) => result,
        Err(_) => Err(AppError::config("MCP TCP bridge thread panicked")),
    }
}

fn tcp_port(config: &AppConfig) -> AppResult<u16> {
    config.port.checked_add(1).ok_or_else(|| {
        AppError::config("cannot derive the MCP TCP port when the HTTP port is 65535")
    })
}

fn tcp_url(port: u16) -> String {
    format!("tcp://{MCP_TCP_HOST}:{port}")
}

async fn serve_tcp_session(stream: TcpStream, context: AppContext) -> AppResult<()> {
    let (reader, writer) = stream.into_split();
    let mut lines = AsyncBufReader::new(reader).lines();
    let mut writer = AsyncBufWriter::new(writer);
    let mut server = McpServer::new(context);

    while let Some(line) = lines
        .next_line()
        .await
        .map_err(|error| AppError::io("read MCP TCP input", error))?
    {
        if line.trim().is_empty() {
            continue;
        }

        let responses = server.handle_line(&line).await;
        for response in responses {
            let payload = serde_json::to_string(&response).map_err(|error| {
                AppError::config(format!("serialize MCP TCP response: {error}"))
            })?;
            writer
                .write_all(payload.as_bytes())
                .await
                .map_err(|error| AppError::io("write MCP TCP output", error))?;
            writer
                .write_all(b"\n")
                .await
                .map_err(|error| AppError::io("write MCP TCP output", error))?;
        }
        writer
            .flush()
            .await
            .map_err(|error| AppError::io("flush MCP TCP output", error))?;
    }

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

struct McpServer {
    context: AppContext,
    negotiated_protocol_version: Option<String>,
    initialized: bool,
}

impl McpServer {
    fn new(context: AppContext) -> Self {
        Self {
            context,
            negotiated_protocol_version: None,
            initialized: false,
        }
    }

    async fn handle_line(&mut self, line: &str) -> Vec<Value> {
        match serde_json::from_str::<Value>(line) {
            Ok(Value::Array(messages)) if messages.is_empty() => {
                vec![error_response(None, -32600, "Invalid Request", None)]
            }
            Ok(Value::Array(messages)) => {
                let mut responses = Vec::new();
                for message in messages {
                    if let Some(response) = self.handle_message(message).await {
                        responses.push(response);
                    }
                }
                responses
            }
            Ok(message) => self.handle_message(message).await.into_iter().collect(),
            Err(error) => vec![error_response(
                None,
                -32700,
                "Parse error",
                Some(json!({ "details": error.to_string() })),
            )],
        }
    }

    async fn handle_message(&mut self, message: Value) -> Option<Value> {
        let Value::Object(object) = message else {
            return Some(error_response(None, -32600, "Invalid Request", None));
        };

        let method = object.get("method").and_then(Value::as_str)?;
        let id = object.get("id").cloned();
        let params = object.get("params").cloned();

        match method {
            "initialize" => {
                let request_id = id?;
                let parsed = parse_params::<InitializeParams>(params);
                Some(match parsed {
                    Ok(params) => self.handle_initialize(request_id, params),
                    Err(message) => error_response(
                        Some(request_id),
                        -32602,
                        "Invalid params",
                        Some(json!({ "details": message })),
                    ),
                })
            }
            "notifications/initialized" => {
                if self.negotiated_protocol_version.is_some() {
                    self.initialized = true;
                }
                None
            }
            "ping" => id.map(|request_id| success_response(request_id, json!({}))),
            _ if !self.initialized => id.map(|request_id| {
                error_response(
                    Some(request_id),
                    -32002,
                    "Server not initialized",
                    Some(json!({
                        "required_sequence": ["initialize", "notifications/initialized"]
                    })),
                )
            }),
            "tools/list" => id.map(|request_id| {
                success_response(request_id, json!({ "tools": tool_definitions() }))
            }),
            "tools/call" => {
                let request_id = id?;
                let parsed = parse_params::<ToolCallParams>(params);
                Some(match parsed {
                    Ok(params) => self.handle_tool_call(request_id, params).await,
                    Err(message) => error_response(
                        Some(request_id),
                        -32602,
                        "Invalid params",
                        Some(json!({ "details": message })),
                    ),
                })
            }
            "notifications/cancelled" => None,
            _ => id.map(|request_id| {
                error_response(Some(request_id), -32601, "Method not found", None)
            }),
        }
    }

    fn handle_initialize(&mut self, request_id: Value, params: InitializeParams) -> Value {
        let negotiated = if SUPPORTED_PROTOCOL_VERSIONS
            .iter()
            .any(|version| *version == params.protocol_version)
        {
            params.protocol_version
        } else {
            SUPPORTED_PROTOCOL_VERSIONS[0].to_string()
        };

        self.negotiated_protocol_version = Some(negotiated.clone());
        self.initialized = false;

        success_response(
            request_id,
            json!({
                "protocolVersion": negotiated,
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    }
                },
                "serverInfo": {
                    "name": SERVER_NAME,
                    "title": SERVER_TITLE,
                    "version": env!("CARGO_PKG_VERSION")
                },
                "instructions": "Use the wardrobe.* tools for live wardrobe state and actions. The Rust backend remains the single source of truth."
            }),
        )
    }

    async fn handle_tool_call(&mut self, request_id: Value, params: ToolCallParams) -> Value {
        let service = self.context.service.clone();

        let result = match params.name.as_str() {
            "wardrobe.health" => tool_success(service.health().await.map(|health| {
                json!({
                    "status": "ok",
                    "item_count": health.item_count,
                    "location_count": health.location_count,
                    "trip_count": health.trip_count
                })
            })),
            "wardrobe.list_items" => tool_success(
                with_args::<ListItemsArgs, _, _>(params.arguments, |args| {
                    let service = service.clone();
                    async move {
                        let items = service
                            .list_items_filtered(ItemFilter {
                                query: args.q,
                                category: args.category,
                                brand: args.brand,
                                season: args.season,
                                current_location_id: args.current_location_id,
                                status: args.status,
                            })
                            .await?;
                        Ok(json!({ "items": items }))
                    }
                })
                .await,
            ),
            "wardrobe.get_item" => tool_success(
                with_args::<ItemIdArgs, _, _>(params.arguments, |args| {
                    let service = service.clone();
                    async move {
                        let item = service
                            .get_item(&args.item_id)
                            .await?
                            .ok_or_else(|| AppError::invalid_argument("item does not exist"))?;
                        Ok(json!({ "item": item }))
                    }
                })
                .await,
            ),
            "wardrobe.create_item" => tool_success(
                with_args::<CreateItemArgs, _, _>(params.arguments, |args| {
                    let service = service.clone();
                    async move {
                        let item = service
                            .create_item(NewItem {
                                name: args.name,
                                category: args.category,
                                subcategory: args.subcategory,
                                brand: args.brand,
                                size: args.size,
                                color_primary: args.color_primary,
                                color_secondary: args.color_secondary,
                                material: args.material,
                                season: args.season,
                                formality: args.formality,
                                status: args.status,
                                current_location_id: args.current_location_id,
                                notes: args.notes,
                            })
                            .await?;
                        Ok(json!({ "item": item }))
                    }
                })
                .await,
            ),
            "wardrobe.list_locations" => tool_success(
                service
                    .list_locations()
                    .await
                    .map(|locations| json!({ "locations": locations })),
            ),
            "wardrobe.create_location" => tool_success(
                with_args::<CreateLocationArgs, _, _>(params.arguments, |args| {
                    let service = service.clone();
                    async move {
                        let location = service
                            .create_location(NewLocation {
                                name: args.name,
                                location_type: args.location_type,
                                parent_id: args.parent_id,
                                notes: args.notes,
                            })
                            .await?;
                        Ok(json!({ "location": location }))
                    }
                })
                .await,
            ),
            "wardrobe.analyze_item_photo" => tool_success(
                with_args::<AnalyzeItemPhotoArgs, _, _>(params.arguments, |args| {
                    let service = service.clone();
                    async move {
                        let suggestion = service.analyze_item_photo_path(&args.image_path).await?;
                        Ok(json!({ "suggestion": suggestion }))
                    }
                })
                .await,
            ),
            "wardrobe.move_item" => tool_success(
                with_args::<MoveItemArgs, _, _>(params.arguments, |args| {
                    let service = service.clone();
                    async move {
                        let result = service
                            .move_item(
                                &args.item_id,
                                MoveItemInput {
                                    to_location_id: args.to_location_id,
                                    reason: args.reason,
                                    notes: args.notes,
                                },
                            )
                            .await?;
                        Ok(json!({
                            "item": result.item,
                            "movement": result.movement
                        }))
                    }
                })
                .await,
            ),
            "wardrobe.get_item_movements" => tool_success(
                with_args::<ItemIdArgs, _, _>(params.arguments, |args| {
                    let service = service.clone();
                    async move {
                        let movements = service.get_item_movements(&args.item_id).await?;
                        Ok(json!({ "movements": movements }))
                    }
                })
                .await,
            ),
            "wardrobe.list_trips" => tool_success(
                service
                    .list_trips()
                    .await
                    .map(|trips| json!({ "trips": trips })),
            ),
            "wardrobe.create_trip" => tool_success(
                with_args::<CreateTripArgs, _, _>(params.arguments, |args| {
                    let service = service.clone();
                    async move {
                        let trip = service
                            .create_trip(NewTrip {
                                name: args.name,
                                destination: args.destination,
                                start_date: args.start_date,
                                end_date: args.end_date,
                                trip_type: args.trip_type,
                                luggage_type: args.luggage_type,
                                notes: args.notes,
                            })
                            .await?;
                        Ok(json!({ "trip": trip }))
                    }
                })
                .await,
            ),
            "wardrobe.update_trip" => tool_success(
                with_args::<UpdateTripArgs, _, _>(params.arguments, |args| {
                    let service = service.clone();
                    async move {
                        let existing = service
                            .get_trip(&args.trip_id)
                            .await?
                            .ok_or_else(|| AppError::invalid_argument("trip does not exist"))?;
                        let trip = service
                            .update_trip(
                                &args.trip_id,
                                UpdateTripInput {
                                    name: args.name.unwrap_or(existing.name),
                                    destination: args.destination.or(existing.destination),
                                    start_date: args.start_date.or(existing.start_date),
                                    end_date: args.end_date.or(existing.end_date),
                                    trip_type: args.trip_type.or(existing.trip_type),
                                    luggage_type: args.luggage_type.or(existing.luggage_type),
                                    notes: args.notes.or(existing.notes),
                                },
                            )
                            .await?;
                        Ok(json!({ "trip": trip }))
                    }
                })
                .await,
            ),
            "wardrobe.get_trip" => tool_success(
                with_args::<TripIdArgs, _, _>(params.arguments, |args| {
                    let service = service.clone();
                    async move {
                        let trip = service
                            .get_trip(&args.trip_id)
                            .await?
                            .ok_or_else(|| AppError::invalid_argument("trip does not exist"))?;
                        Ok(json!({ "trip": trip }))
                    }
                })
                .await,
            ),
            "wardrobe.add_trip_item" => tool_success(
                with_args::<AddTripItemArgs, _, _>(params.arguments, |args| {
                    let service = service.clone();
                    async move {
                        let trip_item = service
                            .add_trip_item(
                                &args.trip_id,
                                NewTripItem {
                                    item_id: args.item_id,
                                    planned_day: args.planned_day,
                                    status: args.status,
                                    notes: args.notes,
                                },
                            )
                            .await?;
                        Ok(json!({ "trip_item": trip_item }))
                    }
                })
                .await,
            ),
            "wardrobe.update_trip_item" => tool_success(
                with_args::<UpdateTripItemArgs, _, _>(params.arguments, |args| {
                    let service = service.clone();
                    async move {
                        let trip_item = service
                            .update_trip_item(
                                &args.trip_id,
                                &args.trip_item_id,
                                UpdateTripItemInput {
                                    planned_day: args.planned_day,
                                    status: args.status,
                                    notes: args.notes,
                                },
                            )
                            .await?;
                        Ok(json!({ "trip_item": trip_item }))
                    }
                })
                .await,
            ),
            "wardrobe.list_trip_items" => tool_success(
                with_args::<TripIdArgs, _, _>(params.arguments, |args| {
                    let service = service.clone();
                    async move {
                        let trip_items = service.list_trip_items(&args.trip_id).await?;
                        Ok(json!({ "trip_items": trip_items }))
                    }
                })
                .await,
            ),
            "wardrobe.remove_trip_item" => tool_success(
                with_args::<RemoveTripItemArgs, _, _>(params.arguments, |args| {
                    let service = service.clone();
                    async move {
                        service
                            .remove_trip_item(&args.trip_id, &args.trip_item_id)
                            .await?;
                        Ok(json!({
                            "removed": true,
                            "trip_id": args.trip_id,
                            "trip_item_id": args.trip_item_id
                        }))
                    }
                })
                .await,
            ),
            _ => {
                return error_response(
                    Some(request_id),
                    -32601,
                    "Unknown tool",
                    Some(json!({ "tool": params.name })),
                );
            }
        };

        success_response(request_id, result)
    }
}

fn parse_params<T: DeserializeOwned>(params: Option<Value>) -> Result<T, String> {
    let value = params.unwrap_or_else(|| json!({}));
    serde_json::from_value(value).map_err(|error| error.to_string())
}

fn parse_arguments<T: DeserializeOwned>(arguments: Option<Value>) -> AppResult<T> {
    let value = arguments.unwrap_or_else(|| json!({}));
    serde_json::from_value(value)
        .map_err(|error| AppError::invalid_argument(format!("invalid tool arguments: {error}")))
}

async fn with_args<T, F, Fut>(arguments: Option<Value>, handler: F) -> AppResult<Value>
where
    T: DeserializeOwned,
    F: FnOnce(T) -> Fut,
    Fut: std::future::Future<Output = AppResult<Value>>,
{
    let parsed = parse_arguments::<T>(arguments)?;
    handler(parsed).await
}

fn tool_success(result: AppResult<Value>) -> Value {
    match result {
        Ok(structured) => tool_result(structured),
        Err(error) => tool_error_from_app_error(error),
    }
}

fn tool_result(structured: Value) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": serde_json::to_string_pretty(&structured).unwrap_or_else(|_| "{}".to_string())
            }
        ],
        "structuredContent": structured
    })
}

fn tool_error_from_app_error(error: AppError) -> Value {
    let (code, message) = match error {
        AppError::InvalidArgument(message) => ("INVALID_ARGUMENT", message),
        AppError::NotInitialized { reason, .. } => ("SERVICE_NOT_READY", reason),
        AppError::Config(message) => ("CONFIG_ERROR", message),
        AppError::Io { context, source } => ("IO_ERROR", format!("{context}: {source}")),
        AppError::Database { context, source } => {
            ("DATABASE_ERROR", format!("{context}: {source}"))
        }
    };

    json!({
        "content": [
            {
                "type": "text",
                "text": message
            }
        ],
        "structuredContent": {
            "error": {
                "code": code,
                "message": message
            }
        },
        "isError": true
    })
}

fn success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn error_response(id: Option<Value>, code: i64, message: &str, data: Option<Value>) -> Value {
    let mut error = Map::new();
    error.insert("code".to_string(), json!(code));
    error.insert("message".to_string(), json!(message));
    if let Some(data) = data {
        error.insert("data".to_string(), data);
    }

    let mut response = Map::new();
    response.insert("jsonrpc".to_string(), json!("2.0"));
    response.insert("id".to_string(), id.unwrap_or(Value::Null));
    response.insert("error".to_string(), Value::Object(error));
    Value::Object(response)
}

fn tool_definitions() -> Vec<Value> {
    vec![
        tool_definition(
            "wardrobe.health",
            "Read wardrobe health counts.",
            json!({ "type": "object", "additionalProperties": false }),
            true,
            false,
            true,
        ),
        tool_definition(
            "wardrobe.list_items",
            "List wardrobe items from the local backend.",
            json!({
                "type": "object",
                "properties": {
                    "q": { "type": "string" },
                    "category": { "type": "string" },
                    "brand": { "type": "string" },
                    "season": { "type": "string" },
                    "current_location_id": { "type": "string" },
                    "status": { "type": "string" }
                }
            }),
            true,
            false,
            true,
        ),
        tool_definition(
            "wardrobe.get_item",
            "Read one wardrobe item by id.",
            json!({
                "type": "object",
                "properties": {
                    "item_id": { "type": "string", "description": "The item id to fetch." }
                },
                "required": ["item_id"]
            }),
            true,
            false,
            true,
        ),
        tool_definition(
            "wardrobe.create_item",
            "Create a wardrobe item in the local backend.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "category": { "type": "string" },
                    "subcategory": { "type": "string" },
                    "brand": { "type": "string" },
                    "size": { "type": "string" },
                    "color_primary": { "type": "string" },
                    "color_secondary": { "type": "string" },
                    "material": { "type": "string" },
                    "season": { "type": "string" },
                    "formality": { "type": "string" },
                    "status": { "type": "string" },
                    "current_location_id": { "type": "string" },
                    "notes": { "type": "string" }
                },
                "required": ["name"]
            }),
            false,
            false,
            false,
        ),
        tool_definition(
            "wardrobe.list_locations",
            "List storage locations from the local backend.",
            json!({ "type": "object", "additionalProperties": false }),
            true,
            false,
            true,
        ),
        tool_definition(
            "wardrobe.create_location",
            "Create a storage location in the local backend.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "location_type": { "type": "string" },
                    "parent_id": { "type": "string" },
                    "notes": { "type": "string" }
                },
                "required": ["name", "location_type"]
            }),
            false,
            false,
            false,
        ),
        tool_definition(
            "wardrobe.analyze_item_photo",
            "Analyze a local item photo with the Codex-backed backend and return item field suggestions.",
            json!({
                "type": "object",
                "properties": {
                    "image_path": { "type": "string", "description": "Absolute or relative path to a local image file." }
                },
                "required": ["image_path"]
            }),
            true,
            false,
            true,
        ),
        tool_definition(
            "wardrobe.move_item",
            "Move an item to another location and record movement history.",
            json!({
                "type": "object",
                "properties": {
                    "item_id": { "type": "string" },
                    "to_location_id": { "type": "string" },
                    "reason": { "type": "string" },
                    "notes": { "type": "string" }
                },
                "required": ["item_id"]
            }),
            false,
            false,
            false,
        ),
        tool_definition(
            "wardrobe.get_item_movements",
            "List recorded movement history for one item.",
            json!({
                "type": "object",
                "properties": {
                    "item_id": { "type": "string" }
                },
                "required": ["item_id"]
            }),
            true,
            false,
            true,
        ),
        tool_definition(
            "wardrobe.list_trips",
            "List trips from the local backend.",
            json!({ "type": "object", "additionalProperties": false }),
            true,
            false,
            true,
        ),
        tool_definition(
            "wardrobe.create_trip",
            "Create a trip in the local backend.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "destination": { "type": "string" },
                    "start_date": { "type": "string" },
                    "end_date": { "type": "string" },
                    "trip_type": { "type": "string" },
                    "luggage_type": { "type": "string" },
                    "notes": { "type": "string" }
                },
                "required": ["name"]
            }),
            false,
            false,
            false,
        ),
        tool_definition(
            "wardrobe.update_trip",
            "Update trip metadata in the local backend.",
            json!({
                "type": "object",
                "properties": {
                    "trip_id": { "type": "string" },
                    "name": { "type": "string" },
                    "destination": { "type": "string" },
                    "start_date": { "type": "string" },
                    "end_date": { "type": "string" },
                    "trip_type": { "type": "string" },
                    "luggage_type": { "type": "string" },
                    "notes": { "type": "string" }
                },
                "required": ["trip_id"]
            }),
            false,
            false,
            false,
        ),
        tool_definition(
            "wardrobe.get_trip",
            "Read one trip by id.",
            json!({
                "type": "object",
                "properties": {
                    "trip_id": { "type": "string" }
                },
                "required": ["trip_id"]
            }),
            true,
            false,
            true,
        ),
        tool_definition(
            "wardrobe.add_trip_item",
            "Add an item to a trip packing list.",
            json!({
                "type": "object",
                "properties": {
                    "trip_id": { "type": "string" },
                    "item_id": { "type": "string" },
                    "planned_day": { "type": "string" },
                    "status": { "type": "string" },
                    "notes": { "type": "string" }
                },
                "required": ["trip_id", "item_id"]
            }),
            false,
            false,
            false,
        ),
        tool_definition(
            "wardrobe.update_trip_item",
            "Update one trip packing entry.",
            json!({
                "type": "object",
                "properties": {
                    "trip_id": { "type": "string" },
                    "trip_item_id": { "type": "string" },
                    "planned_day": { "type": "string" },
                    "status": { "type": "string" },
                    "notes": { "type": "string" }
                },
                "required": ["trip_id", "trip_item_id"]
            }),
            false,
            false,
            false,
        ),
        tool_definition(
            "wardrobe.list_trip_items",
            "List trip packing entries for one trip.",
            json!({
                "type": "object",
                "properties": {
                    "trip_id": { "type": "string" }
                },
                "required": ["trip_id"]
            }),
            true,
            false,
            true,
        ),
        tool_definition(
            "wardrobe.remove_trip_item",
            "Remove one item from a trip packing list.",
            json!({
                "type": "object",
                "properties": {
                    "trip_id": { "type": "string" },
                    "trip_item_id": { "type": "string" }
                },
                "required": ["trip_id", "trip_item_id"]
            }),
            false,
            false,
            false,
        ),
    ]
}

fn tool_definition(
    name: &str,
    description: &str,
    input_schema: Value,
    read_only: bool,
    destructive: bool,
    idempotent: bool,
) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema,
        "annotations": {
            "readOnlyHint": read_only,
            "destructiveHint": destructive,
            "idempotentHint": idempotent,
            "openWorldHint": false
        }
    })
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use crate::app::{AppContext, AppLayout, init_app, open_context};
    use crate::config::{AppConfig, DEFAULT_HOST, DEFAULT_PORT};
    use crate::domain::{NewItem, NewLocation, NewTrip};
    use crate::infra::{CodexItemAnalyzer, MediaStorage};
    use crate::repositories::SqliteWardrobeRepository;
    use crate::services::WardrobeService;

    use super::*;

    #[test]
    fn initialize_and_list_tools_flow() {
        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        let context = runtime.block_on(McpSandbox::new().context());
        let mut server = McpServer::new(context);

        let initialize = runtime.block_on(server.handle_line(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"tester","version":"1.0.0"}}}"#,
        ));
        assert_eq!(initialize.len(), 1);
        assert_eq!(initialize[0]["result"]["protocolVersion"], "2025-06-18");

        let before_ready = runtime
            .block_on(server.handle_line(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#));
        assert_eq!(before_ready[0]["error"]["code"], -32002);

        let ready = runtime.block_on(
            server.handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#),
        );
        assert!(ready.is_empty());

        let listed = runtime
            .block_on(server.handle_line(r#"{"jsonrpc":"2.0","id":3,"method":"tools/list"}"#));
        let tools = listed[0]["result"]["tools"].as_array().expect("tool array");
        assert!(tools.iter().any(|tool| tool["name"] == "wardrobe.health"));
        assert!(
            tools
                .iter()
                .any(|tool| tool["name"] == "wardrobe.move_item")
        );
        assert!(
            tools
                .iter()
                .any(|tool| tool["name"] == "wardrobe.list_trip_items")
        );
    }

    #[test]
    fn health_tool_returns_structured_content() {
        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        let sandbox = McpSandbox::new();
        let context = runtime.block_on(sandbox.context());
        let mut server = initialized_server(context);

        let response = runtime.block_on(server.handle_line(
            r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"wardrobe.health","arguments":{}}}"#,
        ));

        assert_eq!(response[0]["result"]["structuredContent"]["status"], "ok");
        assert_eq!(response[0]["result"]["structuredContent"]["item_count"], 0);
    }

    #[test]
    fn list_items_tool_supports_filter_arguments() {
        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        let sandbox = McpSandbox::new();
        let context = runtime.block_on(async {
            let context = sandbox.context().await;
            let _ = context
                .service
                .create_item(NewItem {
                    name: "Summer Blazer".to_string(),
                    category: Some("Outerwear".to_string()),
                    subcategory: None,
                    brand: Some("Example".to_string()),
                    size: None,
                    color_primary: None,
                    color_secondary: None,
                    material: None,
                    season: Some("Summer".to_string()),
                    formality: None,
                    status: Some("ready".to_string()),
                    current_location_id: None,
                    notes: None,
                })
                .await
                .expect("create item");
            let _ = context
                .service
                .create_item(NewItem {
                    name: "Winter Coat".to_string(),
                    category: Some("Outerwear".to_string()),
                    subcategory: None,
                    brand: Some("Archive".to_string()),
                    size: None,
                    color_primary: None,
                    color_secondary: None,
                    material: None,
                    season: Some("Winter".to_string()),
                    formality: None,
                    status: Some("storage".to_string()),
                    current_location_id: None,
                    notes: None,
                })
                .await
                .expect("create item");
            context
        });

        let mut server = initialized_server(context);
        let response = runtime.block_on(server.handle_line(
            r#"{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"wardrobe.list_items","arguments":{"brand":"Example","season":"Summer","status":"ready"}}}"#,
        ));

        assert_eq!(
            response[0]["result"]["structuredContent"]["items"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            response[0]["result"]["structuredContent"]["items"][0]["name"],
            "Summer Blazer"
        );
    }

    #[test]
    fn move_item_and_list_trip_items_tools_use_service_layer() {
        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        let sandbox = McpSandbox::new();
        let context = runtime.block_on(async {
            let context = sandbox.context().await;

            let item = context
                .service
                .create_item(NewItem {
                    name: "Wool Coat".to_string(),
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
            let location = context
                .service
                .create_location(NewLocation {
                    name: "Hallway Closet".to_string(),
                    location_type: "Closet".to_string(),
                    parent_id: None,
                    notes: None,
                })
                .await
                .expect("create location");
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

            (context, item.id, location.id, trip.id)
        });

        let (context, item_id, location_id, trip_id) = context;
        let mut server = initialized_server(context);

        let move_response = runtime.block_on(server.handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{{"name":"wardrobe.move_item","arguments":{{"item_id":"{item_id}","to_location_id":"{location_id}","reason":"winter"}}}}}}"#
        )));
        assert_eq!(
            move_response[0]["result"]["structuredContent"]["movement"]["to_location_id"],
            location_id
        );

        let add_trip_item = runtime.block_on(server.handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{{"name":"wardrobe.add_trip_item","arguments":{{"trip_id":"{trip_id}","item_id":"{item_id}","status":"planned"}}}}}}"#
        )));
        assert_eq!(
            add_trip_item[0]["result"]["structuredContent"]["trip_item"]["item_id"],
            item_id
        );

        let updated_trip_item = runtime.block_on(server.handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{{"name":"wardrobe.update_trip_item","arguments":{{"trip_id":"{trip_id}","trip_item_id":"{}","status":"packed"}}}}}}"#,
            add_trip_item[0]["result"]["structuredContent"]["trip_item"]["id"].as_str().unwrap()
        )));
        assert_eq!(
            updated_trip_item[0]["result"]["structuredContent"]["trip_item"]["status"],
            "packed"
        );

        let trip_items = runtime.block_on(server.handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{{"name":"wardrobe.list_trip_items","arguments":{{"trip_id":"{trip_id}"}}}}}}"#
        )));
        assert_eq!(
            trip_items[0]["result"]["structuredContent"]["trip_items"][0]["item_id"],
            item_id
        );
        assert_eq!(
            trip_items[0]["result"]["structuredContent"]["trip_items"][0]["status"],
            "packed"
        );
    }

    #[test]
    fn analyze_item_photo_tool_returns_structured_suggestion() {
        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        let sandbox = McpSandbox::new();
        let context = runtime.block_on(sandbox.context_with_fake_codex(
            r#"{
                    "name":"Olive Bomber Jacket",
                    "category":"Outerwear",
                    "subcategory":"Bomber Jacket",
                    "brand":null,
                    "size":null,
                    "color_primary":"Olive",
                    "color_secondary":null,
                    "material":"Nylon",
                    "season":"Fall",
                    "formality":"Casual",
                    "status":null,
                    "notes":"Short zip-front bomber jacket.",
                    "summary":"The image appears to show an olive bomber jacket.",
                    "warnings":[]
                }"#,
        ));
        let image_path = sandbox.root.join("sample.jpg");
        fs::write(&image_path, b"fake-image-bytes").expect("write sample image");

        let mut server = initialized_server(context);
        let response = runtime.block_on(server.handle_line(&format!(
            r#"{{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{{"name":"wardrobe.analyze_item_photo","arguments":{{"image_path":"{}"}}}}}}"#,
            image_path.display()
        )));

        assert_eq!(
            response[0]["result"]["structuredContent"]["suggestion"]["name"],
            "Olive Bomber Jacket"
        );
        assert_eq!(
            response[0]["result"]["structuredContent"]["suggestion"]["category"],
            "Outerwear"
        );
    }

    fn initialized_server(context: AppContext) -> McpServer {
        let mut server = McpServer::new(context);
        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        let _ = runtime.block_on(server.handle_line(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"tester","version":"1.0.0"}}}"#,
        ));
        let _ = runtime.block_on(
            server.handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#),
        );
        server
    }

    struct McpSandbox {
        root: PathBuf,
        data_dir: PathBuf,
    }

    impl McpSandbox {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);

            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let root = env::temp_dir().join(format!(
                "mywardrobehelper-mcp-test-{}-{}",
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

        async fn context_with_fake_codex(&self, payload: &str) -> AppContext {
            let config = AppConfig {
                host: DEFAULT_HOST.to_string(),
                port: DEFAULT_PORT,
                data_dir: self.data_dir.clone(),
            };

            init_app(&config).await.expect("initialize database");

            let layout = AppLayout::from_data_dir(config.data_dir.clone());
            let repository = SqliteWardrobeRepository::new(layout.database_file.clone());
            let script_path = write_fake_codex_script(&self.root, payload);
            let service = WardrobeService::new(
                repository,
                MediaStorage::new(layout.root.clone()),
                CodexItemAnalyzer::with_command(layout.root.join("codex"), script_path),
            );

            AppContext {
                config,
                layout,
                service,
            }
        }
    }

    fn write_fake_codex_script(root: &Path, payload: &str) -> PathBuf {
        let script_path = root.join("fake-codex.sh");
        let script = format!(
            "#!/bin/sh\noutput=\"\"\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"--output-last-message\" ]; then\n    output=\"$2\"\n    shift 2\n    continue\n  fi\n  shift\n done\nprintf '%s' '{}' > \"$output\"\n",
            payload.replace('\'', "'\"'\"'")
        );
        fs::write(&script_path, script).expect("write fake codex script");
        let mut permissions = fs::metadata(&script_path)
            .expect("script metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions).expect("chmod fake codex script");
        script_path
    }

    impl Drop for McpSandbox {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
