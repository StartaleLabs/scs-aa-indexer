CREATE TABLE IF NOT EXISTS pm_user_operations (
    time TIMESTAMPTZ NOT NULL,               -- Required for Timescale hypertables

    chain_id INTEGER NOT NULL,               -- Chain ID

    user_op_hash CHAR(66) NOT NULL,
    user_operation JSONB NOT NULL,
    
    -- Ownership and Access
    org_id VARCHAR(30),
    credential_id VARCHAR(30),

    -- Paymaster Information
    paymaster_mode VARCHAR(20),
    paymaster_id VARCHAR(30),
    fund_type VARCHAR(20),

    -- Status + Source
    status VARCHAR(10) NOT NULL,
    data_source VARCHAR(20) NOT NULL,

    -- Meta fields extracted from JSON
    deducted_user CHAR(42),              -- Address of the user who paid
    actual_gas_cost BIGINT,              -- Cost in wei
    actual_gas_used BIGINT,              -- Gas used
    deducted_amount NUMERIC,             -- Token or native amount deducted
    usd_amount NUMERIC,                  -- USD equivalent amount (optional if calculated)
    native_usd_price NUMERIC,            -- USD price of native token (optional if calculated)

    -- Extended metadata (from PaidGasInTokens / GasBalanceDeducted)
    premium NUMERIC,
    token CHAR(42),
    token_charge NUMERIC,
    applied_markup NUMERIC,
    exchange_rate NUMERIC,


    -- Original metadata (still useful for extensibility)
    metadata JSONB NOT NULL,

    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Create hypertable for timeseries data
SELECT create_hypertable('pm_user_operations', by_range('time'));

-- Create hypertable retention policy
SELECT add_retention_policy('pm_user_operations', drop_after => INTERVAL '1 month');

-- Create indexes for faster querying
CREATE UNIQUE INDEX idx_user_chain_id_user_op_hash
  ON pm_user_operations(chain_id, user_op_hash, time);

---- indexes used by dbt
CREATE INDEX idx_status
  ON pm_user_operations(status);

CREATE INDEX idx_updated_at
  ON pm_user_operations(updated_at);

-- Create trigger to set updated_at timestamp on update
CREATE OR REPLACE FUNCTION trigger_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.UPDATED_AT = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for setting updated_at timestamp
CREATE TRIGGER pm_user_operations_set_updated_at
BEFORE UPDATE ON pm_user_operations
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_updated_at();
