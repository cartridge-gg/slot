-- Fast but limited query that only matches:
-- 1. Direct execute_from_outside_v3 calls
-- 2. execute_from_outside_v3 calls sandwiched in a single VRF call
-- Does NOT catch nested VRF calls or other complex patterns
-- Trade-off: Much faster execution, suitable for long time periods
WITH base_tx AS (
  SELECT
    t.block_date,
    t.transaction_hash,
    t.actual_fee_amount / 1e18 AS fee,
    -- decide which match takes priority
    CASE
      -- execute_from_outside_v3 only call
      WHEN CARDINALITY(t.calldata) > 20 AND t.calldata[11] IN (
        {contract_addresses} 
      ) THEN t.calldata[2]
      -- account for VRF preceeding execute_from_outside_v3 call
      WHEN CARDINALITY(t.calldata) > 20 AND t.calldata[20] IN (
        {contract_addresses} 
      ) THEN t.calldata[11]
      ELSE NULL
    END AS matched_user
  FROM starknet.transactions t
  WHERE
    t.block_time >= TIMESTAMP '{created_at}'
    AND CARDINALITY(t.calldata) > 20
    AND (
      t.calldata[11] IN (
        {contract_addresses} 
      )
      OR
      t.calldata[20] IN (
        {contract_addresses} 
      )
    )
),

prices AS (
  SELECT
    DATE_TRUNC('day', minute) AS time,
    AVG(price) AS price
  FROM prices.usd
  WHERE
    blockchain = 'ethereum'
    AND contract_address = 0xca14007eff0db1f8135f4c25b34de49ab0d42766  -- STRK
  GROUP BY 1
),

daily_stats AS (
  SELECT
    DATE_TRUNC('day', b.block_date) AS day,
    COUNT(DISTINCT b.transaction_hash) AS daily_transactions,
    COUNT(DISTINCT b.matched_user) AS daily_users,
    SUM(b.fee) AS daily_fees_strk,
    SUM(b.fee * p.price) AS daily_fees_usd
  FROM base_tx b
  JOIN prices p
    ON DATE_TRUNC('day', b.block_date) = p.time
  GROUP BY 1
),

overall_totals AS (
  SELECT
    COUNT(DISTINCT transaction_hash) AS overall_transactions,
    COUNT(DISTINCT matched_user) AS overall_unique_users,
    SUM(fee) AS overall_fees_strk,
    SUM(fee * p.price) AS overall_fees_usd
  FROM base_tx b
  JOIN prices p
    ON DATE_TRUNC('day', b.block_date) = p.time
)

SELECT
  d.*,
  o.overall_transactions,
  o.overall_unique_users,
  o.overall_fees_strk,
  o.overall_fees_usd
FROM daily_stats d
CROSS JOIN overall_totals o
ORDER BY d.day;