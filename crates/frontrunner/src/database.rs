//! Database persistence layer

use sqlx::{PgPool, Postgres, QueryBuilder};
use types::{ExecutionResult, SimulationResult, Transaction, Result, Error};
use tracing::{debug, error};
use uuid::Uuid;

/// Database repository for persisting frontrunner data
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Save a transaction to the database
    pub async fn save_transaction(&self, tx: &Transaction) -> Result<()> {
        debug!(tx_id = %tx.id, "Saving transaction to database");

        sqlx::query!(
            r#"
            INSERT INTO transactions (
                id, hash, from_address, to_address, value,
                gas_price, max_fee_per_gas, max_priority_fee_per_gas,
                gas_limit, nonce, data, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (id) DO UPDATE SET
                status = $12,
                updated_at = $14
            "#,
            tx.id,
            format!("{:?}", tx.hash),
            format!("{:?}", tx.from),
            tx.to.map(|a| format!("{:?}", a)),
            tx.value.to_string(),
            tx.gas_price.map(|v| v.to_string()),
            tx.max_fee_per_gas.map(|v| v.to_string()),
            tx.max_priority_fee_per_gas.map(|v| v.to_string()),
            tx.gas_limit.to_string(),
            tx.nonce.as_u64() as i64,
            format!("0x{}", hex::encode(&tx.data.0)),
            tx.status.to_string(),
            tx.created_at,
            tx.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    /// Save a simulation result
    pub async fn save_simulation(&self, sim: &SimulationResult) -> Result<()> {
        debug!(sim_id = %sim.id, "Saving simulation to database");

        let (revenue, cost, profit, profit_pct) = if let Some(ref p) = sim.profitability {
            (
                Some(p.expected_revenue.to_string()),
                Some(p.expected_cost.to_string()),
                Some(p.expected_profit.to_string()),
                Some(p.profit_percentage),
            )
        } else {
            (None, None, None, None)
        };

        sqlx::query!(
            r#"
            INSERT INTO simulations (
                id, original_tx_hash, modified_tx_hash, success, gas_used,
                expected_revenue, expected_cost, expected_profit, profit_percentage,
                error_message, simulation_duration_ms, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            sim.id,
            format!("{:?}", sim.original_tx_hash),
            sim.modified_tx_hash.map(|h| format!("{:?}", h)),
            sim.success,
            sim.gas_used.map(|g| g as i64),
            revenue,
            cost,
            profit,
            profit_pct,
            sim.error_message,
            sim.simulation_duration_ms as i64,
            sim.created_at,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    /// Save an execution result
    pub async fn save_execution(&self, exec: &ExecutionResult) -> Result<()> {
        debug!(exec_id = %exec.id, "Saving execution result to database");

        sqlx::query!(
            r#"
            INSERT INTO execution_results (
                id, simulation_id, submitted_tx_hash, submission_path,
                block_number, gas_used, actual_gas_price, actual_profit,
                success, inclusion_time_ms, error_message, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            exec.id,
            exec.simulation_id,
            format!("{:?}", exec.submitted_tx_hash),
            exec.submission_path.to_string(),
            exec.block_number.map(|n| n as i64),
            exec.gas_used.map(|g| g as i64),
            exec.actual_gas_price.map(|p| p.to_string()),
            exec.actual_profit.map(|p| p.to_string()),
            exec.success,
            exec.inclusion_time_ms.map(|t| t as i64),
            exec.error_message,
            exec.created_at,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    /// Update daily statistics
    pub async fn update_daily_stats(
        &self,
        date: chrono::NaiveDate,
        processed: i64,
        simulated: i64,
        sim_successful: i64,
        submitted: i64,
        confirmed: i64,
        gas_spent: String,
        revenue: String,
        profit: String,
    ) -> Result<()> {
        debug!(date = %date, "Updating daily statistics");

        sqlx::query!(
            r#"
            INSERT INTO daily_statistics (
                date, transactions_processed, simulations_run, simulations_successful,
                transactions_submitted, transactions_confirmed, total_gas_spent,
                total_revenue, total_profit, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            ON CONFLICT (date) DO UPDATE SET
                transactions_processed = daily_statistics.transactions_processed + $2,
                simulations_run = daily_statistics.simulations_run + $3,
                simulations_successful = daily_statistics.simulations_successful + $4,
                transactions_submitted = daily_statistics.transactions_submitted + $5,
                transactions_confirmed = daily_statistics.transactions_confirmed + $6,
                total_gas_spent = (daily_statistics.total_gas_spent::numeric + $7::numeric)::text,
                total_revenue = (daily_statistics.total_revenue::numeric + $8::numeric)::text,
                total_profit = (daily_statistics.total_profit::numeric + $9::numeric)::text,
                updated_at = NOW()
            "#,
            date,
            processed,
            simulated,
            sim_successful,
            submitted,
            confirmed,
            gas_spent,
            revenue,
            profit,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    /// Get today's statistics
    pub async fn get_today_stats(&self) -> Result<DailyStats> {
        let today = chrono::Utc::now().naive_utc().date();

        let stats = sqlx::query_as!(
            DailyStats,
            r#"
            SELECT
                transactions_processed,
                simulations_run,
                simulations_successful,
                transactions_submitted,
                transactions_confirmed,
                total_gas_spent,
                total_revenue,
                total_profit
            FROM daily_statistics
            WHERE date = $1
            "#,
            today
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(stats.unwrap_or_default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct DailyStats {
    pub transactions_processed: Option<i64>,
    pub simulations_run: Option<i64>,
    pub simulations_successful: Option<i64>,
    pub transactions_submitted: Option<i64>,
    pub transactions_confirmed: Option<i64>,
    pub total_gas_spent: Option<String>,
    pub total_revenue: Option<String>,
    pub total_profit: Option<String>,
}
