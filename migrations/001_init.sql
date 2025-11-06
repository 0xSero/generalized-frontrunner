-- Initial database schema for generalized frontrunner

-- Transactions table
CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY,
    hash VARCHAR(66) UNIQUE NOT NULL,
    from_address VARCHAR(42) NOT NULL,
    to_address VARCHAR(42),
    value NUMERIC(78, 0) NOT NULL,
    gas_price NUMERIC(78, 0),
    max_fee_per_gas NUMERIC(78, 0),
    max_priority_fee_per_gas NUMERIC(78, 0),
    gas_limit NUMERIC(78, 0) NOT NULL,
    nonce BIGINT NOT NULL,
    data TEXT,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_tx_hash ON transactions(hash);
CREATE INDEX idx_tx_status ON transactions(status);
CREATE INDEX idx_tx_created_at ON transactions(created_at);

-- Simulations table
CREATE TABLE IF NOT EXISTS simulations (
    id UUID PRIMARY KEY,
    original_tx_hash VARCHAR(66) NOT NULL,
    modified_tx_hash VARCHAR(66),
    success BOOLEAN NOT NULL DEFAULT FALSE,
    gas_used BIGINT,
    expected_revenue NUMERIC(78, 0),
    expected_cost NUMERIC(78, 0),
    expected_profit NUMERIC(78, 0),
    profit_percentage DOUBLE PRECISION,
    error_message TEXT,
    simulation_duration_ms BIGINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_sim_original_tx ON simulations(original_tx_hash);
CREATE INDEX idx_sim_success ON simulations(success);
CREATE INDEX idx_sim_created_at ON simulations(created_at);

-- Execution results table
CREATE TABLE IF NOT EXISTS execution_results (
    id UUID PRIMARY KEY,
    simulation_id UUID NOT NULL,
    submitted_tx_hash VARCHAR(66) UNIQUE NOT NULL,
    submission_path VARCHAR(50) NOT NULL,
    block_number BIGINT,
    gas_used BIGINT,
    actual_gas_price NUMERIC(78, 0),
    actual_profit NUMERIC(78, 0),
    success BOOLEAN NOT NULL DEFAULT FALSE,
    inclusion_time_ms BIGINT,
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    FOREIGN KEY (simulation_id) REFERENCES simulations(id)
);

CREATE INDEX idx_exec_submission_path ON execution_results(submission_path);
CREATE INDEX idx_exec_success ON execution_results(success);
CREATE INDEX idx_exec_block_number ON execution_results(block_number);
CREATE INDEX idx_exec_created_at ON execution_results(created_at);

-- Daily statistics table
CREATE TABLE IF NOT EXISTS daily_statistics (
    date DATE PRIMARY KEY,
    transactions_processed BIGINT DEFAULT 0,
    simulations_run BIGINT DEFAULT 0,
    simulations_successful BIGINT DEFAULT 0,
    transactions_submitted BIGINT DEFAULT 0,
    transactions_confirmed BIGINT DEFAULT 0,
    total_gas_spent NUMERIC(78, 0) DEFAULT 0,
    total_revenue NUMERIC(78, 0) DEFAULT 0,
    total_profit NUMERIC(78, 0) DEFAULT 0,
    avg_latency_ms INT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_daily_stats_date ON daily_statistics(date);
