# MCP Server Template - Rust

A Model Context Protocol (MCP) server template written in Rust, demonstrating how to build an MCP server using **Streamable HTTP** transport instead of STDIO.

This project is based on the tutorial from [paulyu.dev/article/rust-mcp-server-weather-tutorial](https://paulyu.dev/article/rust-mcp-server-weather-tutorial/) but has been adapted to use HTTP transport for greater flexibility in deployment scenarios.

## Overview

This template implements a weather service MCP server that provides:
- Weather alerts for US states
- Weather forecasts based on latitude/longitude coordinates

The server demonstrates how to:
- Build an MCP server in Rust using the [`rmcp`](https://docs.rs/rmcp) library
- Implement HTTP transport with Axum web framework
- Create MCP tools with proper JSON-RPC 2.0 protocol handling
- Integrate with external APIs (National Weather Service)
- Run in a Dockerized development environment

## Features

- **HTTP Transport**: Uses Streamable HTTP instead of STDIO for easier deployment and integration
- **Weather Tools**: 
  - `get_alerts`: Fetch active weather alerts for any US state
  - `get_forecast`: Get detailed weather forecasts using coordinates
- **Production Ready**: Optimized build profiles for development and release
- **DevContainer Support**: Runs in a Docker devcontainer for consistent development environment
- **CORS Enabled**: Ready for web-based clients

## Technology Stack

- **Language**: Rust (Edition 2024)
- **MCP Library**: [`rmcp`](https://docs.rs/rmcp) v0.1.5
- **Web Framework**: Axum v0.7
- **HTTP Client**: reqwest v0.12
- **Runtime**: Tokio (async/await)
- **Logging**: tracing + tracing-subscriber
- **Data Format**: JSON via serde_json

## Getting Started

### Prerequisites

- Docker with DevContainer support (VS Code with Remote-Containers extension recommended)
- Or: Rust 1.75+ toolchain installed locally

### Running in DevContainer

1. Open the project in VS Code
2. When prompted, click "Reopen in Container" (or use Command Palette: "Remote-Containers: Reopen in Container")
3. Wait for the container to build and start
4. Run the server:
   ```bash
   cargo run
   ```

The server will start on `http://0.0.0.0:3000`

### Running Locally

```bash
# Build and run in development mode
cargo run

# Build for production
cargo build --release

# Run the release binary
./target/release/mcp-server-template
```

## Usage

### Testing with MCP Inspector

The easiest way to test the MCP server is using the official MCP Inspector tool. Since this server runs in a Docker container, you'll need to specify the host and allowed origins:

```bash
HOST=0.0.0.0 ALLOWED_ORIGINS=http://localhost:6274 npx @modelcontextprotocol/inspector http://localhost:3000/mcp
```

This will:
1. Start the MCP Inspector web interface at `http://localhost:6274`
2. Connect to your MCP server running in Docker at `http://localhost:3000/mcp`
3. Provide an interactive UI to test all available tools

**Note:** The `HOST=0.0.0.0` and `ALLOWED_ORIGINS` settings are necessary when running the server in Docker to properly handle cross-origin requests from the inspector.

### Manual Testing with curl

Check if the server is running:
```bash
curl http://localhost:3000/mcp
```

### MCP Tool Endpoints

The server implements the following MCP protocol methods:

#### Initialize
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-11-25",
      "capabilities": {},
      "clientInfo": {
        "name": "test-client",
        "version": "1.0.0"
      }
    }
  }'
```

#### List Tools
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list"
  }'
```

#### Get Weather Alerts
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "get_alerts",
      "arguments": {
        "state": "CA"
      }
    }
  }'
```

#### Get Weather Forecast
```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
      "name": "get_forecast",
      "arguments": {
        "latitude": "39.7456",
        "longitude": "-97.0892"
      }
    }
  }'
```

## Project Structure

```
.
├── src/
│   └── main.rs          # Main server implementation
├── Cargo.toml           # Rust dependencies and build configuration
├── Dockerfile           # Docker image definition
├── .devcontainer/       # DevContainer configuration
└── README.md            # This file
```

## Configuration

The server configuration can be modified in [`src/main.rs`](src/main.rs):

- **Port**: Currently set to `3000` (line 381)
- **Bind Address**: `0.0.0.0` for container access (line 381)
- **User Agent**: Configure for NWS API compliance (line 21)
- **Log Level**: Controlled via `RUST_LOG` environment variable

## Development

### Build Profiles

The project includes optimized build profiles in [`Cargo.toml`](Cargo.toml):

- **Development**: Fast compilation with basic optimizations
- **Release**: Maximum performance with LTO and stripped binaries
- **Test**: Balanced optimization for test execution

### Adding New Tools

To add new MCP tools:

1. Define the tool function in the `Weather` impl block using the `#[tool]` attribute
2. Add request/response structs with `serde::Deserialize` and `schemars::JsonSchema` derives
3. Update the `tools/list` handler in [`mcp_handler()`](src/main.rs:259)
4. Add the tool call handling in the `tools/call` match statement

## API Reference

### Available Tools

#### `get_alerts`
Get active weather alerts for a US state.

**Parameters:**
- `state` (string, required): Two-letter US state code (e.g., "CA", "NY")

**Returns:** Formatted text with alert details including event type, affected area, severity, status, and headline.

#### `get_forecast`
Get weather forecast for specific coordinates.

**Parameters:**
- `latitude` (string, required): Latitude in decimal format
- `longitude` (string, required): Longitude in decimal format

**Returns:** Formatted forecast periods with temperature, wind information, and conditions.

## License

MIT License - See LICENSE file for details

## Resources

- [Original Tutorial](https://paulyu.dev/article/rust-mcp-server-weather-tutorial/)
- [MCP Protocol Specification](https://spec.modelcontextprotocol.io/)
- [rmcp Documentation](https://docs.rs/rmcp)
- [National Weather Service API](https://www.weather.gov/documentation/services-web-api)
- [Axum Web Framework](https://docs.rs/axum)

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
