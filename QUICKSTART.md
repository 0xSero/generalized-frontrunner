# Quick Start Guide

Get your generalized frontrunner running in 15 minutes!

## Prerequisites

- Rust 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- PostgreSQL 15+
- Ethereum RPC provider (Alchemy or Infura account)
- $1000+ for initial gas costs

## Step 1: Clone and Configure (5 min)

```bash
# Clone repository
git clone https://github.com/0xSero/generalized-frontrunner.git
cd generalized-frontrunner

# Copy environment template
cp .env.example .env

# Edit .env with your details
nano .env
```

### Required Configuration:
```env
# RPC Providers
ALCHEMY_HTTP_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
ALCHEMY_WS_URL=wss://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
ALCHEMY_API_KEY=your_key_here

# Your Wallet (BE CAREFUL!)
WALLET_PRIVATE_KEY=0xYOUR_PRIVATE_KEY_HERE
WALLET_ADDRESS=0xYOUR_ADDRESS_HERE

# Database
DATABASE_URL=postgresql://postgres:password@localhost:5432/frontrunner
```

## Step 2: Set Up Database (3 min)

```bash
# Install PostgreSQL (if not installed)
# On Ubuntu/Debian:
sudo apt-get install postgresql postgresql-contrib

# On macOS:
brew install postgresql

# Create database
createdb frontrunner

# Migrations run automatically on first start!
```

## Step 3: Build (5 min)

```bash
# Build release version (optimized)
cargo build --release

# This may take 5-10 minutes on first build
# Grab a coffee ☕
```

## Step 4: Configure Strategy (2 min)

Edit `config/default.toml`:

```toml
[filters]
min_value_wei = "100000000000000000"  # 0.1 ETH minimum
min_gas_price_gwei = 10               # Ignore if too low
max_gas_price_gwei = 500              # Ignore if too high

[profitability]
min_profit_wei = "50000000000000000"  # 0.05 ETH minimum profit
min_profit_percentage = 5.0           # 5% profit margin minimum
max_gas_cost_percentage = 70.0        # Max 70% of revenue can be gas

[gas]
max_priority_fee_multiplier = 1.2    # 20% higher priority fee
max_base_fee_multiplier = 1.5        # 50% higher max fee
```

## Step 5: Test Run (Optional but Recommended)

### Run on Testnet First!

```bash
# Use Goerli testnet in .env:
ALCHEMY_HTTP_URL=https://eth-goerli.g.alchemy.com/v2/YOUR_KEY
ALCHEMY_WS_URL=wss://eth-goerli.g.alchemy.com/v2/YOUR_KEY

# Get testnet ETH from faucet:
# https://goerlifaucet.com/

# Run
cargo run --release --bin frontrunner
```

Expected output:
```
INFO Starting Generalized Frontrunner v0.1.0
INFO Configuration loaded environment=development
INFO Connecting to database
INFO Database connected
INFO Database migrations complete
INFO Starting metrics server port=9090
INFO Starting health check server port=8080
INFO Complete Orchestrator starting with all features
INFO Connected to chain chain_id=5
INFO Wallet loaded address=0x...
INFO Nonce manager initialized address=0x... nonce=42
INFO Starting mempool monitor
INFO Subscribing to pending transactions
INFO Starting complete transaction processing loop
```

### Monitor Performance:
```bash
# In another terminal:

# Check health
curl http://localhost:8080/health

# Check metrics
curl http://localhost:9090/metrics

# Watch logs
tail -f logs/frontrunner.log  # if configured
```

## Step 6: Go Live on Mainnet

⚠️ **WARNING**: Only proceed after successful testnet run!

```bash
# Update .env to mainnet:
ALCHEMY_HTTP_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
ALCHEMY_WS_URL=wss://eth-mainnet.g.alchemy.com/v2/YOUR_KEY

# Start with limited capital (update config):
min_value_wei = "10000000000000000"  # 0.01 ETH
min_profit_wei = "5000000000000000"  # 0.005 ETH

# Run
RUST_LOG=info cargo run --release --bin frontrunner
```

## Monitoring

### Prometheus Metrics (`:9090/metrics`)

Key metrics to watch:
- `transactions_processed_total` - Total txs seen
- `simulations_run_total` - Simulations executed
- `profitable_transactions_total` - Profitable opportunities found
- `transactions_submitted_total` - Frontrunning attempts
- `transactions_confirmed_total` - Successful frontrunning
- `gas_spent_wei_total` - Total gas spent
- `profit_earned_wei_total` - Total profit earned

### Health Check (`:8080/health`)

```bash
watch -n 5 'curl -s http://localhost:8080/health | jq'
```

### Database Queries

```sql
-- Today's statistics
SELECT * FROM daily_statistics
WHERE date = CURRENT_DATE;

-- Recent profitable simulations
SELECT * FROM simulations
WHERE success = true
  AND profit_percentage > 5.0
ORDER BY created_at DESC
LIMIT 10;

-- Execution success rate
SELECT
  submission_path,
  COUNT(*) as total,
  SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful,
  ROUND(100.0 * SUM(CASE WHEN success THEN 1 ELSE 0 END) / COUNT(*), 2) as success_rate
FROM execution_results
GROUP BY submission_path;
```

## Expected Behavior

### First Hour:
- ✅ 10,000-50,000 transactions processed
- ✅ 100-500 simulations run
- ✅ 1-10 profitable opportunities found
- ✅ 0-3 successful frontrunning transactions

### First Day:
- ✅ 500K-1M transactions processed
- ✅ 5,000-10,000 simulations
- ✅ 50-200 profitable opportunities
- ✅ 10-50 successful frontruns

### Profitability:
- **Week 1**: Break-even or small loss (learning phase)
- **Week 2-4**: $500-$2,000/week profit
- **Month 2+**: $5,000-$20,000/month profit (with optimization)

## Troubleshooting

### No Transactions Being Processed
```bash
# Check WebSocket connection
RUST_LOG=debug cargo run --release --bin frontrunner

# Should see:
# DEBUG Subscribing to pending transactions
# DEBUG Transaction queued for processing
```

### Simulations Failing
```bash
# Check RPC limits
# Upgrade Alchemy plan if hitting rate limits

# Check simulation timeout
# Increase in config/default.toml:
timeout_ms = 200  # Increase from 100ms
```

### No Profitable Transactions
```bash
# Lower thresholds temporarily:
min_profit_wei = "10000000000000000"  # 0.01 ETH
min_profit_percentage = 2.0           # 2%

# Or wait for high volatility periods
```

### Database Connection Issues
```bash
# Verify PostgreSQL is running
sudo systemctl status postgresql

# Check connection
psql -U postgres -d frontrunner -c "SELECT 1;"

# Reset database if needed
dropdb frontrunner && createdb frontrunner
```

### High Gas Costs, No Profit
```bash
# Reduce gas multipliers:
max_priority_fee_multiplier = 1.1  # Lower from 1.2
max_base_fee_multiplier = 1.3      # Lower from 1.5

# Increase minimum profit:
min_profit_wei = "100000000000000000"  # 0.1 ETH
```

## Optimization Tips

### After First Week:

1. **Analyze Success Rate**:
```sql
SELECT
  DATE(created_at) as date,
  COUNT(*) as attempts,
  SUM(CASE WHEN success THEN 1 ELSE 0 END) as successes,
  ROUND(100.0 * SUM(CASE WHEN success THEN 1 ELSE 0 END) / COUNT(*), 2) as rate
FROM execution_results
GROUP BY DATE(created_at)
ORDER BY date DESC;
```

2. **Tune Gas Prices**: If success rate < 20%, increase multipliers

3. **Adjust Filters**: If too few opportunities, lower min_value_wei

4. **Monitor Competition**: If being outbid, increase multipliers

### After First Month:

1. **Add Flashbots**: Integrate Flashbots for private submission
2. **Multiple Strategies**: Add sandwich attacks, liquidations
3. **Increase Capital**: Scale up min_value_wei
4. **Advanced Filters**: Add ML-based opportunity detection

## Safety & Security

### 🔒 Security Checklist:
- [ ] Use dedicated wallet (not your main wallet!)
- [ ] Store private key in encrypted vault
- [ ] Enable 2FA on RPC provider account
- [ ] Use VPN for RPC connections
- [ ] Monitor for unusual activity
- [ ] Set spending limits
- [ ] Back up database regularly

### 🚨 Kill Switch:
If things go wrong, immediately:
```bash
# Stop the bot
pkill -9 frontrunner

# Check pending transactions
# Cancel if possible through your wallet

# Review logs
grep ERROR logs/frontrunner.log

# Check P&L
psql frontrunner -c "SELECT * FROM daily_statistics WHERE date = CURRENT_DATE;"
```

## Support

- **Issues**: https://github.com/0xSero/generalized-frontrunner/issues
- **Discussions**: https://github.com/0xSero/generalized-frontrunner/discussions
- **Documentation**: See `SCOPE_OF_WORK.md` and `COMPLETION_REPORT.md`

## Next Steps

Once running successfully:
1. Read `COMPLETION_REPORT.md` for advanced features
2. Study `SCOPE_OF_WORK.md` for optimization strategies
3. Join MEV research communities
4. Continuously monitor and optimize

---

**Good luck, and may your transactions always confirm! 🚀**
