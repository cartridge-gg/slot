-- Exhaustive query that:
-- 1. Finds all execute_from_outside_v3 selectors in calldata
-- 2. Handles both normal calls and multi-call VRF helpers
-- 3. Matches all patterns including nested VRF calls
-- Heavy, may timeout on very long ranges
WITH exploded AS (          -- unnest calldata once
  SELECT
    t.transaction_hash,
    t.block_date,
    t.actual_fee_amount / 1e18      AS fee,
    elem                            AS calldata_item,
    idx                             AS calldata_index,
    t.calldata
  FROM starknet.transactions t
  CROSS JOIN UNNEST(t.calldata) WITH ORDINALITY AS u(elem, idx)
  WHERE t.block_time >= TIMESTAMP '{start_time}'
    {end_time_constraint}
),

anchor AS (                 -- normal selector position
  SELECT
    transaction_hash,
    block_date,
    fee,
    calldata,
    calldata_index            AS anchor_index
  FROM exploded
  WHERE calldata_item = 0x03dbc508ba4afd040c8dc4ff8a61113a7bcaf5eae88a6ba27b3c50578b3587e3
),

vrf AS (                    -- first occurrence of the VRF helper contract
  SELECT
    transaction_hash,
    MIN(calldata_index)       AS vrf_index
  FROM exploded
  WHERE calldata_item = 0x051fea4450da9d6aee758bdeba88b2f665bcbf549d2c61421aa724e9ac0ced8f
  GROUP BY transaction_hash
),

anchor_plus AS (            -- combine anchor + optional vrf index
  SELECT
    a.transaction_hash        AS tx_hash,
    a.block_date,
    a.fee,
    a.calldata,
    a.anchor_index,
    v.vrf_index
  FROM anchor a
  LEFT JOIN vrf v
    ON v.transaction_hash = a.transaction_hash
),

target_and_user AS (        -- pick correct offset with COALESCE
  SELECT
    ap.tx_hash                AS transaction_hash,
    ap.block_date,
    ap.fee,
    ap.calldata,
    tgt.calldata_item         AS target_address,
    usr.calldata_item         AS user_address
  FROM anchor_plus ap
  /* target: vrf_index + 3  OR  anchor_index + 8  (fallback) */
  LEFT JOIN exploded tgt
    ON tgt.transaction_hash = ap.tx_hash
   AND tgt.calldata_index =
         COALESCE(ap.vrf_index + 3, ap.anchor_index + 8)

  /* user: always anchor_index − 1 */
  LEFT JOIN exploded usr
    ON usr.transaction_hash = ap.tx_hash
   AND usr.calldata_index = ap.anchor_index - 1

  WHERE tgt.calldata_item IN ({contract_addresses})
),

prices AS (                 -- STRK daily USD price
  SELECT
    DATE_TRUNC('day', minute) AS time,
    AVG(price)               AS price
  FROM prices.usd
  WHERE blockchain = 'ethereum'
    AND contract_address = 0xca14007eff0db1f8135f4c25b34de49ab0d42766
  GROUP BY 1
),

daily_stats AS (            -- per-day metrics
  SELECT
    DATE_TRUNC('day', t.block_date)          AS day,
    COUNT(DISTINCT t.transaction_hash)       AS daily_transactions,
    COUNT(DISTINCT t.user_address)           AS daily_users,
    SUM(t.fee)                               AS daily_fees_strk,
    SUM(t.fee * p.price)                     AS daily_fees_usd
  FROM target_and_user t
  JOIN prices p
    ON DATE_TRUNC('day', t.block_date) = p.time
  GROUP BY 1
),

overall_totals AS (         -- all-time metrics
  SELECT
    COUNT(DISTINCT transaction_hash)         AS overall_transactions,
    COUNT(DISTINCT user_address)             AS overall_unique_users,
    SUM(fee)                                 AS overall_fees_strk,
    SUM(fee * p.price)                       AS overall_fees_usd
  FROM target_and_user t
  JOIN prices p
    ON DATE_TRUNC('day', t.block_date) = p.time
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