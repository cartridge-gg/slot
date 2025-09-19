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

-- --- New helpers for WAU/MAU & new/returning breakdowns ---

first_seen AS (             -- first time each wallet appears (aligned to day/week/month)
  SELECT
    t.user_address,
    MIN(t.block_date)                             AS first_ts,
    DATE_TRUNC('day',   MIN(t.block_date))        AS first_day,
    DATE_TRUNC('week',  MIN(t.block_date))        AS first_week,
    DATE_TRUNC('month', MIN(t.block_date))        AS first_month
  FROM target_and_user t
  WHERE t.user_address IS NOT NULL
  GROUP BY 1
),

daily_user_presence AS (    -- distinct user/day pairs to power rolling windows
  SELECT DISTINCT
    DATE_TRUNC('day', t.block_date) AS day,
    t.user_address
  FROM target_and_user t
  WHERE t.user_address IS NOT NULL
),

days AS (                   -- activity days (switch to a generated calendar for dense dates)
  SELECT DISTINCT DATE_TRUNC('day', block_date) AS day
  FROM target_and_user
),

wau_by_day AS (             -- rolling 7-day (inclusive) unique users
  SELECT
    d.day,
    COUNT(DISTINCT dup.user_address) AS wau
  FROM days d
  LEFT JOIN daily_user_presence dup
    ON dup.day BETWEEN d.day - INTERVAL '6' DAY AND d.day
  GROUP BY 1
),

mau_by_day AS (             -- rolling 30-day (inclusive) unique users
  SELECT
    d.day,
    COUNT(DISTINCT dup.user_address) AS mau
  FROM days d
  LEFT JOIN daily_user_presence dup
    ON dup.day BETWEEN d.day - INTERVAL '29' DAY AND d.day
  GROUP BY 1
),

-- --- Aggregates ---

daily_stats AS (            -- per-day metrics
  SELECT
    DATE_TRUNC('day', t.block_date)          AS day,
    COUNT(DISTINCT t.transaction_hash)       AS daily_transactions,
    COUNT(DISTINCT t.user_address)           AS daily_users,
    COUNT(DISTINCT CASE
      WHEN fs.first_day = DATE_TRUNC('day', t.block_date) THEN t.user_address
    END)                                     AS daily_new_users,
    SUM(t.fee)                               AS daily_fees_strk,
    SUM(t.fee * p.price)                     AS daily_fees_usd
  FROM target_and_user t
  JOIN prices p
    ON DATE_TRUNC('day', t.block_date) = p.time
  LEFT JOIN first_seen fs
    ON fs.user_address = t.user_address
  GROUP BY 1
),

weekly_stats AS (           -- weekly aggregates (ISO week)
  SELECT
    DATE_TRUNC('week', t.block_date)         AS week,
    COUNT(DISTINCT t.transaction_hash)       AS weekly_transactions,
    COUNT(DISTINCT t.user_address)           AS weekly_users,
    COUNT(DISTINCT CASE
      WHEN fs.first_week = DATE_TRUNC('week', t.block_date) THEN t.user_address
    END)                                     AS weekly_new_users,
    SUM(t.fee)                               AS weekly_fees_strk,
    SUM(t.fee * p.price)                     AS weekly_fees_usd
  FROM target_and_user t
  JOIN prices p
    ON DATE_TRUNC('day', t.block_date) = p.time
  LEFT JOIN first_seen fs
    ON fs.user_address = t.user_address
  GROUP BY 1
),

monthly_stats AS (          -- monthly aggregates
  SELECT
    DATE_TRUNC('month', t.block_date)        AS month,
    COUNT(DISTINCT t.transaction_hash)       AS monthly_transactions,
    COUNT(DISTINCT t.user_address)           AS monthly_users,
    COUNT(DISTINCT CASE
      WHEN fs.first_month = DATE_TRUNC('month', t.block_date) THEN t.user_address
    END)                                     AS monthly_new_users,
    SUM(t.fee)                               AS monthly_fees_strk,
    SUM(t.fee * p.price)                     AS monthly_fees_usd
  FROM target_and_user t
  JOIN prices p
    ON DATE_TRUNC('day', t.block_date) = p.time
  LEFT JOIN first_seen fs
    ON fs.user_address = t.user_address
  GROUP BY 1
),

overall_from_base AS (      -- overall totals from base (no nested scalar subqueries)
  SELECT
    COUNT(DISTINCT t.transaction_hash)       AS overall_transactions,
    COUNT(DISTINCT t.user_address)           AS overall_unique_users,
    SUM(t.fee)                               AS overall_fees_strk,
    SUM(t.fee * p.price)                     AS overall_fees_usd
  FROM target_and_user t
  JOIN prices p
    ON DATE_TRUNC('day', t.block_date) = p.time
),

overall_from_daily AS (     -- overall ratio using daily aggregates
  SELECT
    SUM(daily_transactions) AS sum_daily_tx,
    SUM(daily_users)        AS sum_daily_users
  FROM daily_stats
)

SELECT
  d.day,
  d.daily_transactions,
  d.daily_users,
  d.daily_new_users,
  GREATEST(COALESCE(d.daily_users, 0) - COALESCE(d.daily_new_users, 0), 0) AS daily_returning_users,
  (d.daily_transactions / NULLIF(d.daily_users, 0)) AS daily_tx_per_user,
  w7.wau,
  m30.mau,
  d.daily_fees_strk,
  d.daily_fees_usd,
  w.week,
  w.weekly_transactions,
  w.weekly_users,
  w.weekly_new_users,
  GREATEST(COALESCE(w.weekly_users, 0) - COALESCE(w.weekly_new_users, 0), 0) AS weekly_returning_users,
  w.weekly_fees_strk,
  w.weekly_fees_usd,
  m.month,
  m.monthly_transactions,
  m.monthly_users,
  m.monthly_new_users,
  GREATEST(COALESCE(m.monthly_users, 0) - COALESCE(m.monthly_new_users, 0), 0) AS monthly_returning_users,
  m.monthly_fees_strk,
  m.monthly_fees_usd,
  ob.overall_transactions,
  ob.overall_unique_users,
  (ofd.sum_daily_tx / NULLIF(ofd.sum_daily_users, 0)) AS overall_tx_per_daily_user,
  ob.overall_fees_strk,
  ob.overall_fees_usd
FROM daily_stats d
LEFT JOIN wau_by_day   w7  ON w7.day   = d.day
LEFT JOIN mau_by_day   m30 ON m30.day  = d.day
LEFT JOIN weekly_stats w   ON w.week   = DATE_TRUNC('week',  d.day)
LEFT JOIN monthly_stats m  ON m.month  = DATE_TRUNC('month', d.day)
CROSS JOIN overall_from_base  ob
CROSS JOIN overall_from_daily ofd
ORDER BY d.day;