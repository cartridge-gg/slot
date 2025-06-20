-- Exhaustive query that:
-- 1. Finds all execute_from_outside_v3 selectors in calldata
-- 2. Uses each selector as an anchor to locate contract addresses
-- 3. Matches all patterns including nested VRF calls
-- Trade-off: Much slower, may timeout on long time periods
WITH exploded AS (
  SELECT
    t.transaction_hash,
    t.block_date,
    t.actual_fee_amount / 1e18 AS fee,
    elem AS calldata_item,
    idx AS calldata_index,
    t.calldata
  FROM starknet.transactions t
  CROSS JOIN UNNEST(t.calldata) WITH ORDINALITY AS u(elem, idx)
  WHERE t.block_time >= TIMESTAMP '{created_at}'
),

anchor AS (
  SELECT
    transaction_hash,
    block_date,
    fee,
    calldata,
    calldata_index AS anchor_index
  FROM exploded
  -- execute_from_outside_v3 selector
  WHERE calldata_item = 0x03dbc508ba4afd040c8dc4ff8a61113a7bcaf5eae88a6ba27b3c50578b3587e3
),

target_and_user AS (
  SELECT
    a.transaction_hash,
    a.block_date,
    a.fee,
    a.calldata,
    target.calldata_item AS target_address,
    user.calldata_item AS user_address
  FROM anchor a
  LEFT JOIN exploded target
    ON target.transaction_hash = a.transaction_hash
    AND target.calldata_index = a.anchor_index + 8
  LEFT JOIN exploded user
    ON user.transaction_hash = a.transaction_hash
    AND user.calldata_index = a.anchor_index - 1
  WHERE target.calldata_item IN (
    {contract_addresses} 
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
    DATE_TRUNC('day', t.block_date) AS day,
    COUNT(DISTINCT t.transaction_hash) AS daily_transactions,
    COUNT(DISTINCT t.user_address) AS daily_users,
    SUM(t.fee * p.price) AS daily_fees_usd
  FROM target_and_user t
  JOIN prices p
    ON DATE_TRUNC('day', t.block_date) = p.time
  GROUP BY 1
),

overall_totals AS (
  SELECT
    COUNT(DISTINCT transaction_hash) AS overall_transactions,
    COUNT(DISTINCT user_address) AS overall_unique_users,
    SUM(fee * p.price) AS overall_fees_usd
  FROM target_and_user t
  JOIN prices p
    ON DATE_TRUNC('day', t.block_date) = p.time
)

SELECT
  d.*,
  o.overall_transactions,
  o.overall_unique_users,
  o.overall_fees_usd
FROM daily_stats d
CROSS JOIN overall_totals o
ORDER BY d.day;