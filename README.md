# Generalized Frontrunner

A high-performance MEV bot for Ethereum that monitors the mempool, analyzes pending transactions, and executes profitable frontrunning strategies.

## ⚠️ Disclaimer

This software is provided for educational and research purposes only. MEV extraction may be subject to regulatory scrutiny. Use at your own risk. The authors are not responsible for any financial losses or legal consequences.

## Architecture

This project uses a modular Rust workspace architecture:

```
frontrunner/
├── crates/
│   ├── frontrunner/     # Main binary and orchestration
│   ├── mempool/         # Mempool monitoring and transaction ingestion
│   ├── simulator/       # Transaction simulation and profitability calculation
│   ├── executor/        # Transaction building and submission
│   └── types/           # Shared types, models, and utilities
├── config/              # Configuration files
├── migrations/          # Database migrations
└── docs/                # Additional documentation
```

## Features

- **Real-time Mempool Monitoring**: WebSocket-based pending transaction stream
- **Transaction Analysis**: ABI decoding and bytecode parsing
- **EVM Simulation**: Fast REVM-based transaction simulation
- **Multi-Path Submission**: Public mempool, Flashbots, and private relays
- **Profitability Calculation**: Advanced profit/loss estimation
- **Monitoring & Metrics**: Prometheus metrics and structured logging

## Prerequisites

- Rust 1.75+ (2021 edition)
- PostgreSQL 15+
- Ethereum RPC provider (Alchemy, Infura, or self-hosted node)
- 16GB+ RAM recommended
- Low-latency network connection

## Quick Start

### 1. Clone the repository

```bash
git clone https://github.com/0xSero/generalized-frontrunner.git
cd generalized-frontrunner
```

### 2. Set up environment variables

```bash
cp .env.example .env
# Edit .env with your configuration
```

### 3. Set up the database

```bash
# Create PostgreSQL database
createdb frontrunner

# Run migrations
sqlx migrate run
```

### 4. Build the project

```bash
cargo build --release
```

### 5. Run the frontrunner

```bash
cargo run --release --bin frontrunner
```

## Configuration

Configuration is managed through:
1. Environment variables (`.env`)
2. Configuration files (`config/default.toml`)
3. Command-line arguments

See [Configuration Guide](docs/configuration.md) for details.

## Development

### Running tests

```bash
cargo test --workspace
```

### Running with logs

```bash
RUST_LOG=debug cargo run --bin frontrunner
```

### Linting

```bash
cargo clippy --workspace --all-targets
cargo fmt --all -- --check
```

## Performance

Target performance metrics:
- **Throughput**: 1000+ transactions/second
- **Latency**: <200ms end-to-end (detection → submission)
- **Simulation**: <100ms per transaction
- **Uptime**: 99.5%+

## Monitoring

Prometheus metrics are exposed on `:9090/metrics`:
- `transactions_processed_total`
- `simulations_run_total`
- `profitable_transactions_total`
- `transactions_submitted_total`
- `gas_spent_wei_total`
- `profit_earned_wei_total`

Health check endpoint: `:8080/health`

## Security

- Private keys are encrypted at rest
- All sensitive data is logged as `[REDACTED]`
- Rate limiting on external APIs
- Input validation on all external data

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting PRs.

## Resources

- [Scope of Work](SCOPE_OF_WORK.md)
- [Architecture Documentation](docs/architecture.md)
- [API Documentation](docs/api.md)
- [Deployment Guide](docs/deployment.md)

## Support

For issues and questions:
- GitHub Issues: https://github.com/0xSero/generalized-frontrunner/issues
- Discussions: https://github.com/0xSero/generalized-frontrunner/discussions
