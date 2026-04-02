# Downloads Triage Agent

**A production-grade, local-first file organization daemon written in Rust**

Downloads Triage Agent (`dtriage`) is a lightweight Rust background service that monitors your `Downloads` folder, automatically categorizing, renaming, and organizing files using a combination of heuristic rules and optional LLM-powered suggestions. It maintains a local SQLite database for state tracking, supports dry-run mode for safety, and exposes a secure CLI for user interaction.

## Features

- **Zero-overhead background utility** - Uses <10MB RAM when idle
- **Automatic file categorization** - Organizes files by type (Documents, Images, Videos, Music, Archives, Installers, Code)
- **Dry-run mode by default** - Review all actions before they execute
- **SQLite state tracking** - Tracks file hashes to detect duplicates
- **OS-native keyring** - Securely stores API keys (no plaintext secrets)
- **Structured logging** - JSON logs with daily rotation
- **Cross-platform** - Works on Windows, Linux, and macOS

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/downloads-triage.git
cd downloads-triage

# Build release binary
cargo build --release

# The binary will be at target/release/dtriage (or dtriage.exe on Windows)
```

### From Cargo

```bash
cargo install dtriage
```

## Quick Start

### 1. Initialize Configuration

The first run will automatically create a default configuration file:

```bash
dtriage config show
```

### 2. Start the Daemon

```bash
dtriage daemon
```

This starts the background watcher that monitors your Downloads folder.

### 3. Review Pending Actions

In another terminal, review what files have been detected:

```bash
dtriage status
dtriage review
```

### 4. Execute Actions

Once you've reviewed the pending actions, execute them:

```bash
dtriage review --apply
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `dtriage daemon` | Start the watcher daemon |
| `dtriage status` | Show current triage status |
| `dtriage review` | Review pending actions (dry-run) |
| `dtriage review --apply` | Execute pending actions |
| `dtriage config show` | Show current configuration |
| `dtriage config set-api-key <key>` | Store LLM API key |
| `dtriage clean` | Clean up old registry entries |

## Configuration

The configuration file is located at:

- **Windows:** `%AppData%\downloads-triage\config.toml`
- **Linux/macOS:** `~/.config/downloads-triage/config.toml`

### Example Configuration

```toml
# Path to Downloads folder
downloads_dir = "C:\\Users\\YourName\\Downloads"

# Data directory (SQLite, logs)
data_dir = "C:\\Users\\YourName\\AppData\\Local\\downloads-triage\\data"

# Dry-run mode (default: true)
dry_run = true

# Log level (debug, info, warn, error)
log_level = "info"

# LLM configuration (optional)
[llm]
enabled = true
model = "gpt-4"
api_key_service = "openai"

# Categorization rules
[[rules]]
name = "Documents"
extensions = ["pdf", "doc", "docx", "txt"]
destination = "Documents"
priority = 10

[[rules]]
name = "Images"
extensions = ["jpg", "jpeg", "png", "gif"]
destination = "Pictures"
priority = 10
```

## Default Categories

| Category | Extensions | Destination |
|----------|------------|-------------|
| Documents | pdf, doc, docx, txt, rtf | Documents |
| Images | jpg, jpeg, png, gif, bmp, svg, webp | Pictures |
| Videos | mp4, avi, mkv, mov, wmv, flv | Videos |
| Music | mp3, wav, flac, aac, ogg, wma | Music |
| Archives | zip, rar, 7z, tar, gz, bz2 | Archives |
| Installers | exe, msi, dmg, pkg | Installers |
| Code | js, ts, py, rs, go, java, cpp, c, h, cs, php, rb, sh | Code |

## Security

- **API keys** are stored in the OS-native keyring (Windows Credential Manager, macOS Keychain, Linux libsecret)
- **No data leaves your machine** unless you explicitly enable LLM features
- **Path validation** prevents directory traversal attacks
- **Dry-run by default** ensures no accidental file moves

## Performance

| Metric | Value |
|--------|-------|
| Memory (idle) | <10MB RAM |
| File hashing (1MB) | ~5-10ms |
| File hashing (10MB) | ~50-100ms |
| Database writes (WAL mode) | <6ms p99 |

## Troubleshooting

### SQLITE_BUSY errors

Ensure WAL mode is enabled (it is by default). If you still see errors, increase the busy timeout in the database configuration.

### File not detected

- Verify the Downloads folder path is correct in the config
- Check that the daemon is running
- Ensure the file is not a temporary file (`.tmp`, `.part`)

### API key not found

Run `dtriage config set-api-key <your-key>` to store the key in the OS keyring.

### High memory usage

Check for memory leaks. The hasher uses streaming (not loading entire files into memory), so memory should stay low.

## Development

### Running Tests

```bash
cargo test
```

### Running Benchmarks

```bash
cargo bench
```

### Building for Release

```bash
cargo build --release
```

## Project Structure

```
dtriage/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI entry point
│   ├── daemon.rs         # Watcher daemon logic
│   ├── triage/
│   │   ├── mod.rs
│   │   ├── categorizer.rs
│   │   ├── hasher.rs
│   │   └── llm.rs
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── database.rs
│   │   └── models.rs
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── review.rs
│   │   ├── status.rs
│   │   ├── config.rs
│   │   └── clean.rs
│   ├── config/
│   │   ├── mod.rs
│   │   ├── paths.rs
│   │   └── rules.rs
│   ├── security/
│   │   ├── mod.rs
│   │   ├── keyring.rs
│   │   └── validation.rs
│   └── logging/
│       └── mod.rs
├── tests/
│   ├── integration/
│   └── common/
└── benches/
    └── performance.rs
```

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Built with the amazing Rust ecosystem
- Uses [notify](https://github.com/notify-rs/notify) for file watching
- Uses [sqlx](https://github.com/launchbadge/sqlx) for async SQLite
- Uses [clap](https://github.com/clap-rs/clap) for CLI parsing
- Uses [tracing](https://github.com/tokio-rs/tracing) for logging
