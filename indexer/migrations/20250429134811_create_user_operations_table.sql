
CREATE TABLE IF NOT EXISTS pm_user_operations (
    user_op_hash CHAR(66) NOT NULL PRIMARY KEY,
    time TIMESTAMPTZ NOT NULL,
    user_operation JSONB NOT NULL,
    project_id VARCHAR(30) NOT NULL,
    paymaster_mode VARCHAR(20) NOT NULL,
    paymaster_id VARCHAR(30) NOT NULL,
    token_address CHAR(42) TEXT,
    fund_type VARCHAR(20) NOT NULL,
    status VARCHAR(10) NOT NULL,
    data_source VARCHAR(20) TEXT NOT NULL,
    metadata JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create hypertable for timeseries data
SELECT create_hypertable('pm_user_operations', 'time');

CREATE OR REPLACE FUNCTION trigger_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.UPDATED_AT = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER pm_user_operations_set_updated_at
BEFORE UPDATE ON pm_user_operations
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_updated_at();
