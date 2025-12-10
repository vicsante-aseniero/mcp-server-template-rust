use anyhow::Result;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use reqwest;
use rmcp::{
    ServerHandler,
    model::{ServerCapabilities, ServerInfo},
    schemars, tool,
};
use serde_json::Value;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{self, EnvFilter};

const NWS_API_BASE: &str = "https://api.weather.gov";
const USER_AGENT: &str = "weather-app/1.0";

// Data structures for NWS API responses
#[derive(Debug, serde::Deserialize)]
pub struct AlertResponse {
    pub features: Vec<Feature>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Feature {
    pub properties: FeatureProps,
}

#[derive(Debug, serde::Deserialize)]
pub struct FeatureProps {
    pub event: String,
    #[serde(rename = "areaDesc")]
    pub area_desc: String,
    pub severity: String,
    pub status: String,
    pub headline: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PointsRequest {
    #[schemars(description = "latitude of the location in decimal format")]
    pub latitude: String,
    #[schemars(description = "longitude of the location in decimal format")]
    pub longitude: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PointsResponse {
    pub properties: PointsProps,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PointsProps {
    pub forecast: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GridPointsResponse {
    pub properties: GridPointsProps,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GridPointsProps {
    pub periods: Vec<Period>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct Period {
    pub name: String,
    pub temperature: i32,
    #[serde(rename = "temperatureUnit")]
    pub temperature_unit: String,
    #[serde(rename = "windSpeed")]
    pub wind_speed: String,
    #[serde(rename = "windDirection")]
    pub wind_direction: String,
    #[serde(rename = "shortForecast")]
    pub short_forecast: String,
}

// Helper functions for formatting output
fn format_alerts(alerts: &[Feature]) -> String {
    if alerts.is_empty() {
        return "No active alerts found.".to_string();
    }

    let mut result = String::with_capacity(alerts.len() * 200);
    for alert in alerts {
        result.push_str(&format!(
            "Event: {}\nArea: {}\nSeverity: {}\nStatus: {}\nHeadline: {}\n---\n",
            alert.properties.event,
            alert.properties.area_desc,
            alert.properties.severity,
            alert.properties.status,
            alert.properties.headline
        ));
    }
    result
}

fn format_forecast(periods: &[Period]) -> String {
    if periods.is_empty() {
        return "No forecast data available.".to_string();
    }

    let mut result = String::with_capacity(periods.len() * 150);
    for period in periods {
        result.push_str(&format!(
            "Name: {}\nTemperature: {}°{}\nWind: {} {}\nForecast: {}\n---\n",
            period.name,
            period.temperature,
            period.temperature_unit,
            period.wind_speed,
            period.wind_direction,
            period.short_forecast
        ));
    }
    result
}

// Weather service implementation
#[derive(Debug, Clone)]
pub struct Weather {
    client: reqwest::Client,
}

#[tool(tool_box)]
impl Weather {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    async fn make_request<T>(&self, url: &str) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        tracing::info!("Making request to: {}", url);
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        tracing::info!("Received response: {:?}", response);
        match response.status() {
            reqwest::StatusCode::OK => response
                .json::<T>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e)),
            status => Err(format!("Request failed with status: {}", status)),
        }
    }

    #[tool(description = "Get weather alerts for a US state")]
    async fn get_alerts(
        &self,
        #[tool(param)]
        #[schemars(description = "the US state to get alerts for")]
        state: String,
    ) -> String {
        tracing::info!("Received request for weather alerts in state: {}", state);
        let url = format!("{}/alerts/active?area={}", NWS_API_BASE, state);

        match self.make_request::<AlertResponse>(&url).await {
            Ok(alerts) => format_alerts(&alerts.features),
            Err(e) => {
                tracing::error!("Failed to fetch alerts: {}", e);
                "No alerts found or an error occurred.".to_string()
            }
        }
    }

    #[tool(description = "Get forecast using latitude and longitude coordinates")]
    async fn get_forecast(
        &self,
        #[tool(aggr)] PointsRequest {
            latitude,
            longitude,
        }: PointsRequest,
    ) -> String {
        tracing::info!(
            "Received coordinates: latitude = {}, longitude = {}",
            latitude,
            longitude
        );

        let points_url = format!("{}/points/{},{}", NWS_API_BASE, latitude, longitude);
        let points_result = self.make_request::<PointsResponse>(&points_url).await;

        let points = match points_result {
            Ok(points) => points,
            Err(e) => {
                tracing::error!("Failed to fetch points: {}", e);
                return "No forecast found or an error occurred.".to_string();
            }
        };

        match self
            .make_request::<GridPointsResponse>(&points.properties.forecast)
            .await
        {
            Ok(forecast) => format_forecast(&forecast.properties.periods),
            Err(e) => {
                tracing::error!("Failed to fetch forecast: {}", e);
                "No forecast found or an error occurred.".to_string()
            }
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for Weather {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A simple weather forecaster".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

// HTTP handler for MCP requests
async fn mcp_handler(
    State(weather): State<Arc<Weather>>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("Received MCP request: {:?}", payload);

    // Parse the JSON-RPC request
    let method = payload.get("method").and_then(|m| m.as_str());
    let id = payload.get("id").cloned();

    let result = match method {
        Some("initialize") => {
            serde_json::json!({
                "protocolVersion": "2025-11-25",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "mcp-weather-service",
                    "version": "0.1.0"
                }
            })
        }
        Some("tools/list") => {
            serde_json::json!({
                "tools": [
                    {
                        "name": "get_alerts",
                        "description": "Get weather alerts for a US state",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "state": {
                                    "type": "string",
                                    "description": "the US state to get alerts for"
                                }
                            },
                            "required": ["state"]
                        }
                    },
                    {
                        "name": "get_forecast",
                        "description": "Get forecast using latitude and longitude coordinates",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "latitude": {
                                    "type": "string",
                                    "description": "latitude of the location in decimal format"
                                },
                                "longitude": {
                                    "type": "string",
                                    "description": "longitude of the location in decimal format"
                                }
                            },
                            "required": ["latitude", "longitude"]
                        }
                    }
                ]
            })
        }
        Some("tools/call") => {
            let params = payload.get("params").ok_or(StatusCode::BAD_REQUEST)?;
            let tool_name = params
                .get("name")
                .and_then(|n| n.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            let arguments = params.get("arguments").ok_or(StatusCode::BAD_REQUEST)?;

            let content = match tool_name {
                "get_alerts" => {
                    let state = arguments
                        .get("state")
                        .and_then(|s| s.as_str())
                        .ok_or(StatusCode::BAD_REQUEST)?;
                    weather.get_alerts(state.to_string()).await
                }
                "get_forecast" => {
                    let latitude = arguments
                        .get("latitude")
                        .and_then(|l| l.as_str())
                        .ok_or(StatusCode::BAD_REQUEST)?;
                    let longitude = arguments
                        .get("longitude")
                        .and_then(|l| l.as_str())
                        .ok_or(StatusCode::BAD_REQUEST)?;
                    let request = PointsRequest {
                        latitude: latitude.to_string(),
                        longitude: longitude.to_string(),
                    };
                    weather.get_forecast(request).await
                }
                _ => return Err(StatusCode::NOT_FOUND),
            };

            serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": content
                    }
                ]
            })
        }
        // Handle notifications (they don't need responses)
        Some(method) if method.starts_with("notifications/") => {
            tracing::info!("Received notification: {}", method);
            return Ok(Json(serde_json::json!({})));
        }
        _ => {
            tracing::warn!("Unknown method: {:?}", method);
            return Err(StatusCode::METHOD_NOT_ALLOWED);
        }
    };

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    });

    Ok(Json(response))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting MCP Weather Service with HTTP transport");

    // Create weather service
    let weather = Arc::new(Weather::new());

    // Create Axum router
    let app = Router::new()
        .route("/mcp", get(|| async { "MCP Weather Service is running" }))
        .route("/mcp", post(mcp_handler))
        .layer(CorsLayer::permissive())
        .with_state(weather);

    // Start server
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("MCP Weather Service listening on http://{}", addr);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");

    Ok(())
}
