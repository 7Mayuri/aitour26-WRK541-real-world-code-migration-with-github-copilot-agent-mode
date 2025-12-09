# Weather API - Rust Migration

This is a Rust migration of the Python FastAPI weather API using the actix-web framework.

## Project Structure

```
rust-app/
├── Cargo.toml          # Rust project configuration and dependencies
├── Makefile            # Build and run commands
├── README.md           # This file
├── src/
│   └── main.rs         # Main application file (scaffolding stage)
└── weather.json        # Weather data (to be added)
```

## Dependencies

- **actix-web**: High-performance web framework for Rust
- **serde**: Serialization/deserialization library
- **serde_json**: JSON support for serde
- **tokio**: Async runtime

## Building the Project

To build the Rust project, run:

```bash
cargo build
```

Or using the Makefile:

```bash
make build
```

## Running the Project

To run the application in development mode:

```bash
cargo run
```

Or using the Makefile:

```bash
make run
```

The server will start at `http://127.0.0.1:8000`

## Testing

To run tests:

```bash
cargo test
```

Or using the Makefile:

```bash
make test
```

## Code Quality

Check code for issues:

```bash
cargo check
```

Format code:

```bash
cargo fmt
```

Run clippy (linter):

```bash
cargo clippy
```

## Next Steps

The scaffolding is now ready. The following endpoints need to be implemented:

1. `GET /` - Redirect to API documentation
2. `GET /countries` - List all available countries
3. `GET /countries/{country}/{city}/{month}` - Get monthly weather averages

The weather data will be loaded from `weather.json` and serialized using serde.
