# Scope of Work: Generalized Ethereum Frontrunner Development

**Project Name:** Generalized MEV Frontrunner System
**Version:** 1.0
**Date:** 2025-11-06
**Status:** Draft

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Project Objectives](#project-objectives)
3. [Technical Requirements](#technical-requirements)
4. [System Architecture](#system-architecture)
5. [Development Phases](#development-phases)
6. [Deliverables](#deliverables)
7. [Timeline & Milestones](#timeline--milestones)
8. [Resource Requirements](#resource-requirements)
9. [Budget Estimate](#budget-estimate)
10. [Risk Assessment](#risk-assessment)
11. [Success Criteria](#success-criteria)
12. [Assumptions & Constraints](#assumptions--constraints)

---

## Executive Summary

This document outlines the scope of work for developing a production-grade generalized frontrunner system for Ethereum mainnet. The system will monitor the mempool for profitable transactions, analyze and replicate them with modified parameters, simulate profitability, and submit optimized frontrunning transactions via multiple submission paths including Flashbots.

**Project Duration:** 12-16 weeks
**Estimated Budget:** $150,000 - $250,000 (development only)
**Team Size:** 3-5 engineers
**Target:** Minimum Viable Product (MVP) → Production-Ready System

---

## Project Objectives

### Primary Objectives

1. **Build a fully functional generalized frontrunner** capable of:
   - Real-time mempool monitoring
   - Transaction analysis and parameter extraction
   - Address replacement in transaction calldata
   - Local transaction simulation and profitability calculation
   - Competitive gas price calculation
   - Multi-path transaction submission

2. **Achieve operational profitability** within 3 months of deployment:
   - Target: Net positive ROI after gas costs
   - Minimum viable profit: $10,000/month
   - Stretch goal: $50,000+/month

3. **Maintain competitive performance**:
   - Transaction analysis latency: <50ms
   - Simulation latency: <100ms
   - End-to-end latency (detection → submission): <200ms

### Secondary Objectives

1. Implement basic obfuscation techniques for operational security
2. Build monitoring and analytics dashboard
3. Establish Flashbots reputation scoring system
4. Create automated alert system for critical failures

---

## Technical Requirements

### 3.1 Core Functional Requirements

#### FR-1: Mempool Monitoring Service
- **Description:** Real-time subscription to pending Ethereum transactions
- **Requirements:**
  - WebSocket connection to Ethereum RPC provider
  - Handle 1000+ transactions per second
  - Automatic reconnection on connection loss
  - Transaction filtering based on configurable criteria
  - Queue management with priority handling

#### FR-2: Transaction Analysis Engine
- **Description:** Parse and extract parameters from pending transactions
- **Requirements:**
  - ABI decoding for known contract interfaces
  - Bytecode disassembly for unknown contracts
  - Address extraction (PUSH20, PUSH32 detection)
  - Function selector identification
  - Calldata parsing and parameter extraction
  - Contract interaction detection (DEX swaps, NFT mints, etc.)

#### FR-3: Address Replacement System
- **Description:** Modify transaction parameters to redirect profits
- **Requirements:**
  - Intelligent address substitution in calldata
  - Bytecode-level address replacement
  - Handle multiple address types (EOA, contracts)
  - Preserve transaction validity after modification
  - Support for complex multi-call transactions

#### FR-4: Simulation Engine
- **Description:** Execute modified transactions in local forked environment
- **Requirements:**
  - Mainnet state forking at current block
  - Fast EVM execution (REVM-based)
  - State change tracking
  - Balance/profit calculation
  - Gas estimation
  - Support for eth_call, eth_estimateGas
  - Batch simulation capabilities

#### FR-5: Profitability Calculator
- **Description:** Determine if frontrunning is economically viable
- **Requirements:**
  - Calculate expected revenue from modified transaction
  - Calculate total gas cost (base fee + priority fee)
  - Factor in opportunity cost
  - Apply configurable profit threshold
  - Risk assessment (simulation confidence scoring)

#### FR-6: Gas Price Optimizer
- **Description:** Calculate competitive gas prices for frontrunning
- **Requirements:**
  - Parse original transaction gas parameters
  - Implement EIP-1559 pricing logic
  - Calculate optimal maxFeePerGas and maxPriorityFeePerGas
  - Dynamic premium calculation based on profitability
  - Protection against overpaying for unprofitable trades

#### FR-7: Transaction Builder
- **Description:** Construct and sign frontrunning transactions
- **Requirements:**
  - Build EIP-1559 transactions
  - Transaction signing with secure key management
  - Nonce management and tracking
  - Support for contract deployment (if needed for strategies)
  - Transaction encoding/serialization

#### FR-8: Multi-Path Submission System
- **Description:** Submit transactions via multiple channels
- **Requirements:**
  - **Public Mempool:** Standard RPC broadcast
  - **Flashbots:** Bundle creation and submission
  - **Private Relays:** Integration with alternative MEV relays
  - Parallel submission to multiple paths
  - Submission result tracking
  - Retry logic with exponential backoff

#### FR-9: Transaction Monitoring
- **Description:** Track submitted transactions and outcomes
- **Requirements:**
  - Monitor transaction inclusion status
  - Track block confirmations
  - Calculate actual profit/loss
  - Record success/failure rates
  - Gas usage tracking
  - Flashbots reputation monitoring

### 3.2 Non-Functional Requirements

#### NFR-1: Performance
- Transaction processing: 500+ transactions/second
- Mempool → decision latency: <200ms (95th percentile)
- Simulation time: <100ms per transaction
- System uptime: 99.5%+

#### NFR-2: Reliability
- Automatic restart on crashes
- Graceful degradation on RPC failures
- Data persistence for audit trails
- No transaction loss during operation

#### NFR-3: Security
- Secure private key storage (HSM or encrypted vault)
- Rate limiting on external API calls
- Input validation on all external data
- Audit logging for all transactions
- Protected configuration management

#### NFR-4: Scalability
- Horizontal scaling for analysis workers
- Support for multiple concurrent strategies
- Configurable resource limits
- Efficient memory management

#### NFR-5: Observability
- Structured logging (JSON format)
- Metrics collection (Prometheus compatible)
- Distributed tracing (optional)
- Health check endpoints
- Performance profiling support

---

## System Architecture

### 4.1 High-Level Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                    External Services Layer                     │
├───────────────────────────────────────────────────────────────┤
│  • Ethereum RPC Providers (Alchemy/Infura)                    │
│  • Flashbots Relay                                             │
│  • Alternative MEV Relays                                      │
└─────────────────────────┬─────────────────────────────────────┘
                          │
┌─────────────────────────▼─────────────────────────────────────┐
│                   Mempool Monitor Service                      │
├───────────────────────────────────────────────────────────────┤
│  • WebSocket subscription manager                              │
│  • Transaction ingestion queue                                 │
│  • Basic filtering logic                                       │
└─────────────────────────┬─────────────────────────────────────┘
                          │
┌─────────────────────────▼─────────────────────────────────────┐
│                   Transaction Analyzer                         │
├───────────────────────────────────────────────────────────────┤
│  • Calldata parser                                             │
│  • ABI decoder                                                 │
│  • Address extractor                                           │
│  • Strategy classifier                                         │
└─────────────────────────┬─────────────────────────────────────┘
                          │
                   [Interesting?]
                          │
              Yes ┌───────┴────────┐ No
                  │                │
                  ▼                ▼
    ┌─────────────────────┐   [Discard]
    │  Replication Engine │
    ├─────────────────────┤
    │ • Address replacer  │
    │ • Param modifier    │
    └──────────┬──────────┘
               │
    ┌──────────▼──────────────────────────────────────────┐
    │              Simulation Environment                  │
    ├─────────────────────────────────────────────────────┤
    │  • REVM-based EVM                                    │
    │  • State forking                                     │
    │  • Transaction execution                             │
    │  • Profitability calculator                          │
    └──────────┬──────────────────────────────────────────┘
               │
         [Profitable?]
               │
      Yes ┌────┴─────┐ No
          │          │
          ▼          ▼
    ┌─────────┐ [Discard + Log]
    │ Builder │
    ├─────────┤
    │ • Gas   │
    │ • Sign  │
    │ • Nonce │
    └────┬────┘
         │
    ┌────▼──────────────────────────────────────────────┐
    │          Submission Orchestrator                   │
    ├───────────────────────────────────────────────────┤
    │  • Multi-path routing                              │
    │  • Parallel submission                             │
    │  • Result tracking                                 │
    └────┬───────────────────────────────────────────────┘
         │
    ┌────▼──────────────────────────────────────────────┐
    │          Monitoring & Analytics                    │
    ├───────────────────────────────────────────────────┤
    │  • Success/failure tracking                        │
    │  • P&L calculation                                 │
    │  • Metrics collection                              │
    │  • Alerting                                        │
    └───────────────────────────────────────────────────┘
```

### 4.2 Technology Stack

#### Language: Rust
**Rationale:** Performance-critical application requiring low latency, memory safety, and concurrent processing.

#### Core Dependencies
```toml
[dependencies]
# Ethereum interaction
ethers = { version = "2.0", features = ["ws", "abigen", "rustls"] }
alloy = "0.1"  # Modern alternative

# EVM simulation
revm = "3.5"
revm-inspectors = "0.1"

# Async runtime
tokio = { version = "1.35", features = ["full", "tracing"] }

# Flashbots
ethers-flashbots = "0.9"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database (for persistence)
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }

# Metrics
prometheus = "0.13"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Configuration
config = "0.13"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Cryptography
ethers-signers = { version = "2.0", features = ["aws", "ledger"] }
```

#### Infrastructure
- **Database:** PostgreSQL 15+ (transaction history, P&L tracking)
- **Monitoring:** Prometheus + Grafana
- **Logging:** JSON structured logs → Loki (optional)
- **Deployment:** Docker + Docker Compose (initial), Kubernetes (scaling)

---

## Development Phases

### Phase 1: Foundation & Infrastructure (Weeks 1-3)

#### Objectives
- Set up development environment
- Establish core infrastructure
- Implement basic mempool monitoring

#### Tasks

**1.1 Project Setup**
- [ ] Initialize Rust workspace with proper module structure
- [ ] Set up Git repository with branching strategy
- [ ] Configure CI/CD pipeline (GitHub Actions)
- [ ] Set up development/staging/production environments
- [ ] Create configuration management system

**1.2 RPC Infrastructure**
- [ ] Integrate Alchemy/Infura RPC providers
- [ ] Implement WebSocket connection manager with auto-reconnect
- [ ] Build connection health monitoring
- [ ] Implement rate limiting and request queuing
- [ ] Create RPC failover logic (primary/backup providers)

**1.3 Mempool Monitor Service**
- [ ] Build WebSocket subscription to pending transactions
- [ ] Implement transaction ingestion queue (bounded channel)
- [ ] Create basic transaction filtering (by value, gas, etc.)
- [ ] Build transaction deduplication logic
- [ ] Add metrics collection (transactions/second, queue depth)

**1.4 Database Setup**
- [ ] Design database schema for transactions, simulations, results
- [ ] Set up PostgreSQL with migrations (sqlx)
- [ ] Implement transaction repository pattern
- [ ] Create database connection pool
- [ ] Build basic CRUD operations

**1.5 Logging & Monitoring**
- [ ] Configure structured logging (tracing-subscriber)
- [ ] Set up Prometheus metrics endpoint
- [ ] Create basic Grafana dashboards
- [ ] Implement health check endpoint
- [ ] Set up log aggregation (optional)

#### Deliverables
- ✅ Functional mempool monitoring service
- ✅ Database schema and connection layer
- ✅ Basic metrics and logging
- ✅ CI/CD pipeline

#### Success Criteria
- Successfully receive and process 1000+ pending transactions/second
- 99.9% WebSocket uptime over 24 hours
- All metrics visible in Grafana dashboard

---

### Phase 2: Transaction Analysis & Parsing (Weeks 4-6)

#### Objectives
- Build transaction analysis engine
- Implement calldata parsing
- Create address extraction logic

#### Tasks

**2.1 ABI Decoder**
- [ ] Integrate ethers ABI decoding
- [ ] Build contract interface registry (Uniswap V2/V3, Sushiswap, etc.)
- [ ] Implement function selector identification
- [ ] Create parameter extraction logic
- [ ] Handle edge cases (malformed calldata)

**2.2 Bytecode Analyzer**
- [ ] Build EVM bytecode disassembler
- [ ] Implement opcode parser
- [ ] Create PUSH20/PUSH32 detection logic
- [ ] Extract addresses from bytecode
- [ ] Build contract fingerprinting system

**2.3 Transaction Classifier**
- [ ] Identify transaction types (DEX swap, NFT mint, etc.)
- [ ] Calculate transaction "interestingness" score
- [ ] Filter for profitable patterns
- [ ] Build pattern matching rules engine
- [ ] Create configurable strategy filters

**2.4 Address Extractor**
- [ ] Extract sender, recipient, contract addresses
- [ ] Identify token addresses
- [ ] Extract LP pool addresses
- [ ] Build address relationship graph
- [ ] Validate extracted addresses

**2.5 Testing & Validation**
- [ ] Unit tests for each parser component
- [ ] Integration tests with real mempool data
- [ ] Benchmark parsing performance
- [ ] Create test fixtures from mainnet transactions
- [ ] Validate parsing accuracy (>95% success rate)

#### Deliverables
- ✅ Transaction analysis engine
- ✅ ABI decoder and calldata parser
- ✅ Address extraction system
- ✅ Transaction classification rules
- ✅ Test suite with >80% coverage

#### Success Criteria
- Successfully parse 95%+ of mainnet transactions
- Parsing latency <20ms per transaction (p95)
- Correctly identify profitable transaction patterns

---

### Phase 3: Replication & Simulation (Weeks 7-9)

#### Objectives
- Build transaction replication logic
- Implement EVM simulation
- Create profitability calculator

#### Tasks

**3.1 Address Replacement Engine**
- [ ] Build calldata address replacement logic
- [ ] Implement bytecode address substitution
- [ ] Handle multi-address transactions
- [ ] Preserve transaction validity checks
- [ ] Create replacement verification system

**3.2 Transaction Modifier**
- [ ] Modify transaction sender
- [ ] Adjust gas parameters
- [ ] Update nonce
- [ ] Recalculate transaction hash
- [ ] Handle special cases (multicall, batch transactions)

**3.3 REVM Integration**
- [ ] Set up REVM simulation environment
- [ ] Implement mainnet state forking
- [ ] Build transaction execution wrapper
- [ ] Create state inspection utilities
- [ ] Implement balance tracking

**3.4 Simulation Engine**
- [ ] Execute modified transactions in REVM
- [ ] Track state changes (balances, storage)
- [ ] Calculate gas usage
- [ ] Detect simulation failures (reverts)
- [ ] Build simulation result analysis

**3.5 Profitability Calculator**
- [ ] Calculate revenue from state changes
- [ ] Compute gas costs (base + priority fees)
- [ ] Factor in slippage and price impact
- [ ] Apply profit thresholds
- [ ] Generate confidence scores

**3.6 Performance Optimization**
- [ ] Implement simulation caching
- [ ] Parallelize simulations
- [ ] Optimize state forking
- [ ] Reduce memory allocations
- [ ] Profile and optimize hot paths

#### Deliverables
- ✅ Transaction replication system
- ✅ REVM-based simulation engine
- ✅ Profitability calculator
- ✅ Performance benchmarks
- ✅ Simulation accuracy validation

#### Success Criteria
- Simulation latency <100ms per transaction (p95)
- Profitability calculation accuracy >90%
- Successfully replicate and simulate 80%+ of identified transactions
- Handle 50+ parallel simulations

---

### Phase 4: Transaction Building & Submission (Weeks 10-12)

#### Objectives
- Build transaction construction system
- Implement gas price optimization
- Create multi-path submission

#### Tasks

**4.1 Gas Price Optimizer**
- [ ] Implement EIP-1559 gas calculation
- [ ] Build dynamic priority fee calculator
- [ ] Create competitive bidding logic
- [ ] Implement gas price capping
- [ ] Build gas market analyzer

**4.2 Transaction Builder**
- [ ] Construct EIP-1559 transactions
- [ ] Implement transaction signing
- [ ] Build nonce management system
- [ ] Create transaction verification
- [ ] Handle edge cases (nonce gaps, etc.)

**4.3 Key Management**
- [ ] Implement secure key storage (encrypted vault)
- [ ] Build key rotation system
- [ ] Support hardware wallet signing (Ledger/Trezor)
- [ ] Implement AWS KMS integration (optional)
- [ ] Create key backup procedures

**4.4 Public Mempool Submission**
- [ ] Build standard RPC transaction broadcast
- [ ] Implement retry logic
- [ ] Create submission result tracking
- [ ] Handle RPC errors gracefully
- [ ] Build submission queue

**4.5 Flashbots Integration**
- [ ] Integrate ethers-flashbots
- [ ] Build bundle creation logic
- [ ] Implement Flashbots transaction submission
- [ ] Create reputation monitoring
- [ ] Build bundle simulation verification

**4.6 Alternative Relay Integration**
- [ ] Research available MEV relays
- [ ] Implement relay abstraction layer
- [ ] Add support for 2-3 additional relays
- [ ] Build relay health monitoring
- [ ] Create relay selection logic

**4.7 Multi-Path Orchestrator**
- [ ] Build parallel submission system
- [ ] Implement path prioritization
- [ ] Create submission result aggregation
- [ ] Build submission analytics
- [ ] Implement adaptive routing

#### Deliverables
- ✅ Transaction building system
- ✅ Gas price optimizer
- ✅ Multi-path submission orchestrator
- ✅ Flashbots integration
- ✅ Secure key management

#### Success Criteria
- Successfully submit transactions to 3+ paths
- Gas price optimization within 5% of optimal
- Transaction submission latency <50ms
- Zero key compromise incidents

---

### Phase 5: Monitoring, Analytics & Optimization (Weeks 13-14)

#### Objectives
- Build comprehensive monitoring
- Implement P&L tracking
- Create alerting system

#### Tasks

**5.1 Result Tracking**
- [ ] Monitor submitted transaction status
- [ ] Track inclusion in blocks
- [ ] Calculate confirmation times
- [ ] Record actual gas used
- [ ] Build transaction receipt processor

**5.2 P&L Calculator**
- [ ] Track revenue from successful frontrunning
- [ ] Calculate actual gas costs
- [ ] Compute net profit/loss per transaction
- [ ] Build cumulative P&L tracking
- [ ] Generate daily/weekly/monthly reports

**5.3 Metrics & Dashboards**
- [ ] Expand Prometheus metrics (success rate, profit, latency)
- [ ] Build comprehensive Grafana dashboards
- [ ] Create real-time performance views
- [ ] Implement historical trend analysis
- [ ] Build strategy-specific metrics

**5.4 Alerting System**
- [ ] Configure Prometheus AlertManager
- [ ] Create critical alerts (crashes, losses, RPC failures)
- [ ] Build warning alerts (high latency, low success rate)
- [ ] Implement notification channels (Slack, Discord, PagerDuty)
- [ ] Test alert delivery

**5.5 Analytics Engine**
- [ ] Build opportunity analysis (missed vs. captured)
- [ ] Calculate strategy performance metrics
- [ ] Identify optimization opportunities
- [ ] Generate competitive analysis reports
- [ ] Build ML/AI preparation (data collection for future)

#### Deliverables
- ✅ Comprehensive monitoring system
- ✅ P&L tracking and reporting
- ✅ Alerting infrastructure
- ✅ Analytics dashboards
- ✅ Performance reports

#### Success Criteria
- 100% transaction result tracking
- Real-time P&L visibility
- Alert delivery <30 seconds
- Dashboard load time <2 seconds

---

### Phase 6: Testing, Security & Deployment (Weeks 15-16)

#### Objectives
- Comprehensive testing
- Security audit
- Production deployment

#### Tasks

**6.1 Testing**
- [ ] Unit tests for all components (>80% coverage)
- [ ] Integration tests (end-to-end flows)
- [ ] Load testing (sustained 1000 tx/s)
- [ ] Chaos engineering (failure injection)
- [ ] Regression testing

**6.2 Security**
- [ ] Code security review
- [ ] Dependency vulnerability scan
- [ ] Key management audit
- [ ] API security testing
- [ ] Create security incident response plan

**6.3 Documentation**
- [ ] System architecture documentation
- [ ] API documentation
- [ ] Deployment guide
- [ ] Operational runbook
- [ ] Troubleshooting guide

**6.4 Deployment Preparation**
- [ ] Create production configuration
- [ ] Set up production infrastructure
- [ ] Configure monitoring and alerting
- [ ] Prepare rollback procedures
- [ ] Create deployment checklist

**6.5 Testnet Validation**
- [ ] Deploy to Goerli/Sepolia testnet
- [ ] Run for 7 days continuous
- [ ] Validate all components
- [ ] Fix any discovered issues
- [ ] Performance tuning

**6.6 Mainnet Deployment**
- [ ] Deploy to production environment
- [ ] Start with limited capital (<$10K)
- [ ] Monitor closely for 48 hours
- [ ] Gradually increase capital allocation
- [ ] Optimize based on real performance

#### Deliverables
- ✅ Comprehensive test suite
- ✅ Security audit report
- ✅ Complete documentation
- ✅ Production deployment
- ✅ Operational runbook

#### Success Criteria
- All tests passing
- Zero critical security vulnerabilities
- Successful testnet run (7 days)
- Successful mainnet deployment
- Net positive P&L within 7 days

---

## Deliverables

### Code Deliverables

1. **Source Code Repository**
   - Complete Rust codebase
   - Modular architecture
   - Well-documented code
   - Git history with meaningful commits

2. **Configuration Files**
   - Environment-specific configs (dev, staging, prod)
   - Secret management templates
   - Infrastructure as Code (Docker Compose / K8s manifests)

3. **Database Schema**
   - Migration scripts
   - Seed data for testing
   - Backup/restore procedures

4. **Test Suite**
   - Unit tests (>80% coverage)
   - Integration tests
   - Performance benchmarks
   - Test fixtures and mocks

### Documentation Deliverables

1. **Technical Documentation**
   - System architecture document
   - API documentation (if applicable)
   - Database schema documentation
   - Component interaction diagrams

2. **Operational Documentation**
   - Deployment guide
   - Configuration guide
   - Monitoring and alerting guide
   - Troubleshooting guide
   - Security best practices

3. **User Documentation**
   - Getting started guide
   - Strategy configuration guide
   - Dashboard user guide
   - FAQ

### Infrastructure Deliverables

1. **Deployment Infrastructure**
   - Docker images
   - Docker Compose files
   - Kubernetes manifests (optional)
   - CI/CD pipeline configuration

2. **Monitoring Infrastructure**
   - Grafana dashboards (JSON exports)
   - Prometheus alerting rules
   - Log aggregation configuration

3. **Database Infrastructure**
   - PostgreSQL setup scripts
   - Backup automation
   - Monitoring queries

---

## Timeline & Milestones

### Overview Timeline: 16 Weeks

```
Week 1-3:   [████████████] Foundation & Infrastructure
Week 4-6:   [████████████] Transaction Analysis
Week 7-9:   [████████████] Replication & Simulation
Week 10-12: [████████████] Building & Submission
Week 13-14: [████████] Monitoring & Analytics
Week 15-16: [████████] Testing & Deployment
```

### Detailed Milestone Schedule

| Milestone | Week | Deliverable | Success Criteria |
|-----------|------|-------------|------------------|
| **M1: Foundation** | 3 | Mempool monitoring service | Process 1000+ tx/s, 99.9% uptime |
| **M2: Analysis** | 6 | Transaction parser | Parse 95%+ of transactions |
| **M3: Simulation** | 9 | Simulation engine | <100ms latency, 90% accuracy |
| **M4: Submission** | 12 | Multi-path submitter | Submit to 3+ paths successfully |
| **M5: Monitoring** | 14 | Analytics dashboard | Real-time P&L tracking |
| **M6: Production** | 16 | Mainnet deployment | Net positive P&L within 1 week |

### Critical Path

```
Mempool Monitor → Transaction Analysis → Simulation Engine →
Transaction Builder → Submission System → Production Deploy
```

**Critical Dependencies:**
- Cannot build simulation without analysis
- Cannot submit without building transactions
- Cannot deploy without monitoring

### Buffer & Contingency

- **2-week buffer** included for unforeseen issues
- **Risk mitigation:** Parallel work streams where possible
- **Contingency:** Additional 4 weeks available if needed (total 20 weeks)

---

## Resource Requirements

### 8.1 Team Structure

#### Core Team (Required)

**1. Senior Blockchain Engineer (Lead)**
- **Role:** Technical lead, architecture, core development
- **Skills:** Rust expert, EVM deep knowledge, MEV experience
- **Time:** Full-time (16 weeks)
- **Responsibilities:**
  - System architecture design
  - Core simulation engine development
  - Code review and quality assurance
  - Technical decision making

**2. Backend Engineer (Rust)**
- **Role:** Infrastructure, mempool monitoring, submission system
- **Skills:** Rust, async programming, distributed systems
- **Time:** Full-time (16 weeks)
- **Responsibilities:**
  - Mempool monitoring service
  - Transaction submission orchestrator
  - Database integration
  - Performance optimization

**3. Smart Contract / EVM Engineer**
- **Role:** Transaction analysis, bytecode parsing, simulation
- **Skills:** Solidity, EVM internals, assembly
- **Time:** Full-time (12 weeks)
- **Responsibilities:**
  - Transaction parser development
  - Address extraction logic
  - Contract interaction analysis
  - Simulation result validation

#### Supporting Team (Recommended)

**4. DevOps Engineer**
- **Role:** Infrastructure, deployment, monitoring
- **Skills:** Docker, K8s, Prometheus, Grafana, CI/CD
- **Time:** Part-time (8 weeks, 50% allocation)
- **Responsibilities:**
  - Infrastructure setup
  - CI/CD pipeline
  - Monitoring configuration
  - Production deployment

**5. QA / Security Engineer**
- **Role:** Testing, security audit, code review
- **Skills:** Security testing, penetration testing, Rust
- **Time:** Part-time (4 weeks, 50% allocation)
- **Responsibilities:**
  - Security audit
  - Test plan creation
  - Vulnerability assessment
  - Key management review

### 8.2 Hardware & Infrastructure

#### Development Environment (Per Engineer)
- **Laptop/Workstation:** 16GB+ RAM, SSD
- **Development RPC:** Alchemy/Infura free tier initially

#### Staging Environment
- **Server:** 8 vCPU, 16GB RAM, 200GB SSD
- **Database:** 4 vCPU, 8GB RAM, 100GB SSD
- **Cost:** ~$200/month (AWS/DigitalOcean)

#### Production Environment (Initial)
- **Application Server:** 16 vCPU, 32GB RAM, 500GB NVMe SSD
- **Database Server:** 8 vCPU, 16GB RAM, 500GB SSD
- **Monitoring Server:** 4 vCPU, 8GB RAM, 200GB SSD
- **Cost:** ~$800/month

#### RPC Services
- **Provider:** Alchemy Growth or Infura Growth
- **Cost:** $200-$500/month
- **Alternative:** Self-hosted node (requires additional 12TB storage)

### 8.3 Software & Services

| Service | Purpose | Cost (Monthly) |
|---------|---------|----------------|
| **GitHub** | Code repository | $0 (public) / $4 per user |
| **Alchemy/Infura** | RPC provider | $200-$500 |
| **PostgreSQL Cloud** | Database (optional) | $0 (self-hosted) / $100 |
| **Monitoring Stack** | Prometheus/Grafana | $0 (self-hosted) |
| **Domain & SSL** | Production domain | $15 |
| **Logging Service** | Loki / Papertrail | $0-$50 |
| **Alert Service** | PagerDuty (optional) | $0-$41 |
| **Total** | | **$415-$706/month** |

---

## Budget Estimate

### 9.1 Development Costs (One-Time)

| Item | Rate | Duration | Cost |
|------|------|----------|------|
| **Senior Blockchain Engineer** | $150-$200/hr | 640 hours (16 weeks) | $96,000-$128,000 |
| **Backend Engineer (Rust)** | $120-$150/hr | 640 hours (16 weeks) | $76,800-$96,000 |
| **Smart Contract Engineer** | $120-$150/hr | 480 hours (12 weeks) | $57,600-$72,000 |
| **DevOps Engineer** | $100-$130/hr | 160 hours (8 weeks, 50%) | $16,000-$20,800 |
| **QA/Security Engineer** | $100-$130/hr | 80 hours (4 weeks, 50%) | $8,000-$10,400 |
| **Subtotal (Labor)** | | | **$254,400-$327,200** |

**Adjusted Estimate (Team Coordination Overhead):**
- **Low Estimate:** $254,400 × 0.9 = **$228,960**
- **High Estimate:** $327,200 × 1.1 = **$359,920**

### 9.2 Infrastructure Costs (4 Months)

| Item | Monthly | Duration | Cost |
|------|---------|----------|------|
| **Development Infrastructure** | $100 | 4 months | $400 |
| **Staging Environment** | $200 | 3 months | $600 |
| **Production Environment** | $800 | 1 month | $800 |
| **RPC Services** | $350 | 4 months | $1,400 |
| **Monitoring & Services** | $100 | 4 months | $400 |
| **Subtotal (Infrastructure)** | | | **$3,600** |

### 9.3 Miscellaneous Costs

| Item | Cost |
|------|------|
| **External Security Audit** (optional) | $15,000-$30,000 |
| **Legal/Compliance Review** (optional) | $5,000-$10,000 |
| **Contingency Buffer** (10%) | $25,000-$35,000 |
| **Subtotal (Misc)** | **$45,000-$75,000** |

### 9.4 Total Development Budget

| Scenario | Labor | Infrastructure | Miscellaneous | **Total** |
|----------|-------|----------------|---------------|-----------|
| **Minimum** | $228,960 | $3,600 | $45,000 | **$277,560** |
| **Expected** | $280,000 | $3,600 | $55,000 | **$338,600** |
| **Maximum** | $359,920 | $3,600 | $75,000 | **$438,520** |

### 9.5 Operational Budget (Post-Launch, Monthly)

| Category | Cost |
|----------|------|
| **Infrastructure** | $800-$1,500 |
| **RPC Services** | $500-$5,000 |
| **Monitoring & Tools** | $100-$500 |
| **Maintenance (Engineer, 25%)** | $8,000-$12,000 |
| **Gas Costs** | $10,000-$500,000+ |
| **Total (Monthly)** | **$19,400-$519,000+** |

**Note:** Gas costs are highly variable and depend on:
- Market volatility
- Number of attempted frontrunning transactions
- Success rate
- Competitive gas prices

**Conservative Estimate:** $50,000/month in gas costs
**Competitive Estimate:** $200,000-$500,000/month in gas costs

---

## Risk Assessment

### 10.1 Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Simulation Inaccuracy** | High | High | Extensive testing against mainnet data, gradual capital ramp |
| **Performance Bottlenecks** | Medium | High | Early performance testing, profiling, optimization sprints |
| **RPC Provider Failures** | Medium | High | Multi-provider failover, self-hosted node backup |
| **EVM Edge Cases** | Medium | Medium | Comprehensive test suite, fuzzing, mainnet validation |
| **Key Compromise** | Low | Critical | HSM/KMS integration, security audit, key rotation |
| **Database Failures** | Low | Medium | Automated backups, replication, monitoring |

### 10.2 Economic Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Unprofitable Operations** | Medium | Critical | Start with low capital, strict profit thresholds, kill switch |
| **High Gas Costs** | High | High | Dynamic gas price limits, profitability gates |
| **Increased Competition** | High | High | Continuous optimization, low-latency infrastructure |
| **Market Conditions** | Medium | Medium | Diversified strategies, adaptive thresholds |
| **Flashbots Reputation Loss** | Low | Medium | Conservative submission, bundle validation |

### 10.3 Operational Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Team Member Departure** | Low | High | Knowledge documentation, pair programming, code reviews |
| **Scope Creep** | Medium | Medium | Strict scope management, change control process |
| **Timeline Delays** | Medium | Medium | Buffer time included, agile sprint planning |
| **Production Incidents** | Medium | High | Comprehensive monitoring, alerting, runbooks |

### 10.4 Regulatory Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **MEV Regulation** | Low | High | Legal review, compliance monitoring, exit strategy |
| **KYC/AML Requirements** | Low | Medium | Use compliant exchanges, maintain records |
| **Smart Contract Liability** | Low | Medium | Legal disclaimer, insurance (optional) |

### 10.5 Risk Management Strategy

1. **Risk Monitoring:** Weekly risk review meetings
2. **Incident Response:** Documented procedures for each critical risk
3. **Kill Switch:** Ability to immediately halt operations if losses exceed threshold
4. **Capital Limits:** Graduated capital deployment (start $10K → $50K → $100K+)
5. **Continuous Testing:** Ongoing validation against mainnet data

---

## Success Criteria

### 11.1 Technical Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **System Uptime** | 99.5%+ | Monthly availability |
| **Transaction Processing** | 1000+ tx/s | Sustained throughput |
| **End-to-End Latency** | <200ms (p95) | Detection → submission |
| **Simulation Accuracy** | >90% | Predicted vs. actual profit |
| **Parsing Success Rate** | >95% | Successfully parsed transactions |
| **Code Coverage** | >80% | Unit + integration tests |

### 11.2 Economic Success Metrics

| Metric | Target | Timeline |
|--------|--------|----------|
| **First Successful Frontrun** | Within 48 hours | Post-mainnet launch |
| **Net Positive P&L** | Within 7 days | Post-mainnet launch |
| **Monthly Profit** | $10,000+ | Month 1 |
| **Monthly Profit** | $50,000+ | Month 3 (stretch) |
| **ROI on Development** | Break-even | Within 6 months |
| **Gas Cost Efficiency** | <70% of revenue | Ongoing |

### 11.3 Operational Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Flashbots Success Rate** | >30% | Bundles landed / submitted |
| **Alert Response Time** | <5 minutes | Incident detection → response |
| **Documentation Completeness** | 100% | All components documented |
| **Security Incidents** | 0 | Key compromises or breaches |
| **Team Velocity** | 40+ story points/sprint | Agile sprint planning |

### 11.4 Go/No-Go Decision Points

**Milestone 3 (Week 9): Simulation Engine**
- **Go Criteria:** Simulation accuracy >85%, latency <150ms
- **No-Go:** Consider pivot to simpler MEV strategies or extend development

**Milestone 6 (Week 16): Production Launch**
- **Go Criteria:** All tests passing, successful testnet run, documented procedures
- **No-Go:** Delay mainnet launch, address critical issues

**Month 1 Post-Launch:**
- **Continue:** Net positive P&L or clear path to profitability
- **Pivot:** Adjust strategies, optimize parameters, reduce costs
- **Shutdown:** Consistent losses, unresolvable technical issues

---

## Assumptions & Constraints

### 12.1 Assumptions

1. **Market Access:**
   - Ethereum mainnet remains accessible and operational
   - RPC providers maintain service quality
   - Flashbots or equivalent MEV infrastructure remains available

2. **Technical:**
   - REVM simulation provides sufficient accuracy
   - Rust ecosystem libraries remain stable and maintained
   - Target frontrunning patterns remain viable

3. **Team:**
   - Engineers have required skill levels
   - Team members available for full duration
   - Effective collaboration and communication

4. **Economic:**
   - MEV opportunities continue to exist at current levels
   - Gas prices remain within historical ranges
   - Competition doesn't drastically increase during development

5. **Regulatory:**
   - No major regulatory changes affecting MEV operations
   - Current legal ambiguity continues
   - No protocol-level changes to prevent frontrunning

### 12.2 Constraints

1. **Technical Constraints:**
   - Must operate within Ethereum block time (~12 seconds)
   - Limited by RPC provider rate limits
   - Dependent on external infrastructure (RPC, Flashbots)

2. **Budget Constraints:**
   - Development budget cap: $450,000
   - Monthly operational budget (pre-gas): $15,000
   - Initial capital for gas: $50,000-$100,000

3. **Timeline Constraints:**
   - Target delivery: 16 weeks (4 months)
   - Hard deadline: 20 weeks (5 months)
   - Cannot delay past Q2 2025 due to market conditions

4. **Resource Constraints:**
   - Maximum team size: 5 engineers
   - Single production environment initially
   - Limited monitoring/alerting budget

5. **Operational Constraints:**
   - 24/7 operation required post-launch
   - Must maintain <1 hour incident response time
   - Requires on-call rotation for production support

### 12.3 Out of Scope

The following items are **explicitly out of scope** for this project:

1. **Advanced Strategies:**
   - Sandwich attacks (separate project)
   - Cross-chain MEV
   - Flashloan-based strategies
   - Liquidation bots

2. **Infrastructure:**
   - Self-hosted Ethereum node (Phase 1)
   - Kubernetes deployment
   - Multi-region deployment
   - High-availability database cluster

3. **Features:**
   - Web-based user interface
   - Mobile application
   - Public API
   - Machine learning / AI optimization

4. **Compliance:**
   - Legal entity formation
   - Formal compliance program
   - Third-party audits (beyond security)

5. **Scaling:**
   - Multi-strategy orchestration
   - White-label solution
   - Commercial licensing

**Note:** Out-of-scope items may be considered for future phases based on initial success.

---

## Approval & Sign-Off

### Document Review

| Role | Name | Date | Signature |
|------|------|------|-----------|
| **Project Sponsor** | | | |
| **Technical Lead** | | | |
| **Product Owner** | | | |
| **Finance Approver** | | | |

### Change Management

**Scope changes require:**
1. Written change request
2. Impact analysis (timeline, budget, resources)
3. Stakeholder review and approval
4. Updated scope document (version increment)

**Version History:**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-11-06 | System | Initial scope of work |

---

## Appendix A: Detailed Task Breakdown

### Phase 1: Foundation & Infrastructure

#### 1.1 Project Setup (Week 1)
```
Estimated Effort: 40 hours

Tasks:
- [ ] Create Rust workspace with cargo workspace.toml (2h)
- [ ] Design module structure (services, models, utils, etc.) (4h)
- [ ] Initialize Git repository with .gitignore (1h)
- [ ] Set up branch protection rules (main, develop, feature/*) (2h)
- [ ] Configure GitHub Actions workflows (CI, test, lint) (8h)
- [ ] Create development/staging/production configs (4h)
- [ ] Set up environment variable management (3h)
- [ ] Document project structure in README.md (4h)
- [ ] Set up pre-commit hooks (rustfmt, clippy) (2h)
- [ ] Initial team onboarding documentation (4h)
- [ ] Code review guidelines and PR templates (2h)
- [ ] Set up project management board (Jira/GitHub Projects) (4h)
```

#### 1.2 RPC Infrastructure (Week 1-2)
```
Estimated Effort: 60 hours

Tasks:
- [ ] Research and select RPC providers (Alchemy, Infura, QuickNode) (4h)
- [ ] Implement RPC provider abstraction trait (6h)
- [ ] Integrate Alchemy WebSocket connection (8h)
- [ ] Integrate Infura WebSocket connection (8h)
- [ ] Build connection manager with auto-reconnect (12h)
- [ ] Implement exponential backoff for reconnection (4h)
- [ ] Build connection health monitoring (heartbeat) (6h)
- [ ] Implement rate limiting per provider (8h)
- [ ] Create request queue with prioritization (6h)
- [ ] Build RPC failover logic (primary → backup) (8h)
- [ ] Unit tests for connection manager (6h)
- [ ] Integration tests with real RPC endpoints (4h)
```

[Continue with similar detailed breakdowns for all phases...]

---

## Appendix B: Database Schema

### Transaction Table
```sql
CREATE TABLE transactions (
    id BIGSERIAL PRIMARY KEY,
    tx_hash VARCHAR(66) UNIQUE NOT NULL,
    from_address VARCHAR(42) NOT NULL,
    to_address VARCHAR(42),
    value NUMERIC(78, 0) NOT NULL,
    gas_price NUMERIC(78, 0),
    max_fee_per_gas NUMERIC(78, 0),
    max_priority_fee_per_gas NUMERIC(78, 0),
    gas_limit BIGINT NOT NULL,
    nonce BIGINT NOT NULL,
    data TEXT,
    status VARCHAR(20) NOT NULL, -- pending, simulated, submitted, confirmed, failed
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_tx_status ON transactions(status);
CREATE INDEX idx_tx_created_at ON transactions(created_at);
```

### Simulation Table
```sql
CREATE TABLE simulations (
    id BIGSERIAL PRIMARY KEY,
    original_tx_hash VARCHAR(66) NOT NULL,
    modified_tx_hash VARCHAR(66),
    simulation_result JSONB NOT NULL,
    expected_profit NUMERIC(78, 0),
    gas_estimate BIGINT,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    simulation_duration_ms INT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    FOREIGN KEY (original_tx_hash) REFERENCES transactions(tx_hash)
);

CREATE INDEX idx_sim_original_tx ON simulations(original_tx_hash);
CREATE INDEX idx_sim_success ON simulations(success);
```

### Execution Results Table
```sql
CREATE TABLE execution_results (
    id BIGSERIAL PRIMARY KEY,
    simulation_id BIGINT NOT NULL,
    submitted_tx_hash VARCHAR(66) UNIQUE NOT NULL,
    submission_path VARCHAR(50) NOT NULL, -- public, flashbots, relay_name
    block_number BIGINT,
    gas_used BIGINT,
    actual_gas_price NUMERIC(78, 0),
    actual_profit NUMERIC(78, 0),
    success BOOLEAN NOT NULL,
    inclusion_time_ms INT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    FOREIGN KEY (simulation_id) REFERENCES simulations(id)
);

CREATE INDEX idx_exec_submission_path ON execution_results(submission_path);
CREATE INDEX idx_exec_success ON execution_results(success);
CREATE INDEX idx_exec_block_number ON execution_results(block_number);
```

### Daily Statistics Table
```sql
CREATE TABLE daily_statistics (
    date DATE PRIMARY KEY,
    transactions_processed BIGINT DEFAULT 0,
    simulations_run BIGINT DEFAULT 0,
    simulations_successful BIGINT DEFAULT 0,
    transactions_submitted BIGINT DEFAULT 0,
    transactions_confirmed BIGINT DEFAULT 0,
    total_gas_spent NUMERIC(78, 0) DEFAULT 0,
    total_profit NUMERIC(78, 0) DEFAULT 0,
    avg_latency_ms INT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

---

## Appendix C: Configuration Template

### config.toml
```toml
[environment]
name = "production"
log_level = "info"

[rpc]
primary_provider = "alchemy"
backup_provider = "infura"
websocket_timeout_seconds = 60
max_reconnect_attempts = 10
rate_limit_requests_per_second = 25

[rpc.providers.alchemy]
http_url = "${ALCHEMY_HTTP_URL}"
ws_url = "${ALCHEMY_WS_URL}"
api_key = "${ALCHEMY_API_KEY}"

[rpc.providers.infura]
http_url = "${INFURA_HTTP_URL}"
ws_url = "${INFURA_WS_URL}"
api_key = "${INFURA_API_KEY}"

[mempool]
queue_size = 10000
worker_threads = 4
transaction_timeout_seconds = 30

[filters]
min_value_wei = "100000000000000000"  # 0.1 ETH
min_gas_price_gwei = 10
max_gas_price_gwei = 500

[simulation]
enabled = true
timeout_ms = 100
max_parallel_simulations = 50
fork_block_offset = 0  # 0 = latest block

[profitability]
min_profit_wei = "50000000000000000"  # 0.05 ETH
min_profit_percentage = 5.0
max_gas_cost_percentage = 70.0

[gas]
max_priority_fee_multiplier = 1.2
max_base_fee_multiplier = 1.5
absolute_max_gwei = 500

[submission]
enabled_paths = ["flashbots", "public"]
flashbots_enabled = true
flashbots_relay_url = "https://relay.flashbots.net"
max_submission_attempts = 3

[submission.flashbots]
signing_key = "${FLASHBOTS_SIGNING_KEY}"
min_reputation_score = 100

[wallet]
private_key = "${WALLET_PRIVATE_KEY}"  # Encrypted or from vault
address = "${WALLET_ADDRESS}"

[database]
url = "${DATABASE_URL}"
max_connections = 10
min_connections = 2
connection_timeout_seconds = 30

[monitoring]
metrics_port = 9090
health_check_port = 8080
enable_tracing = false

[alerting]
enabled = true
slack_webhook_url = "${SLACK_WEBHOOK_URL}"
critical_loss_threshold_eth = "1.0"
```

---

## Appendix D: API Endpoints (Internal)

### Health Check
```
GET /health
Response: 200 OK
{
  "status": "healthy",
  "uptime_seconds": 86400,
  "version": "1.0.0"
}
```

### Metrics
```
GET /metrics
Response: 200 OK (Prometheus format)
# HELP transactions_processed_total Total transactions processed
# TYPE transactions_processed_total counter
transactions_processed_total 12345
...
```

### Status
```
GET /status
Response: 200 OK
{
  "mempool": {
    "connected": true,
    "transactions_per_second": 850,
    "queue_depth": 234
  },
  "simulation": {
    "active_simulations": 23,
    "average_latency_ms": 87
  },
  "submission": {
    "pending_submissions": 5,
    "flashbots_reputation": 1234
  },
  "profitability": {
    "today_profit_eth": "2.45",
    "today_gas_spent_eth": "1.23",
    "today_net_profit_eth": "1.22"
  }
}
```

---

## Appendix E: Monitoring Dashboards

### Dashboard 1: System Overview
- Uptime gauge
- Transactions processed (counter)
- Transactions per second (gauge)
- Queue depth (gauge)
- RPC connection status (gauge)
- Error rate (gauge)

### Dashboard 2: Performance Metrics
- End-to-end latency histogram
- Simulation latency histogram
- Submission latency histogram
- CPU usage (gauge)
- Memory usage (gauge)
- Network throughput (gauge)

### Dashboard 3: Economic Metrics
- Total profit (counter)
- Total gas spent (counter)
- Net profit (gauge)
- Profit per transaction (gauge)
- Success rate (gauge)
- Flashbots bundle success rate (gauge)

### Dashboard 4: Transaction Pipeline
- Transactions received (counter)
- Transactions parsed (counter)
- Transactions simulated (counter)
- Profitable transactions (counter)
- Transactions submitted (counter)
- Transactions confirmed (counter)
- Pipeline funnel visualization

---

*End of Scope of Work Document*
