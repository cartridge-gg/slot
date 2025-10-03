WITH contract_set AS (
  SELECT ARRAY[
    {contract_addresses}
  ] AS addrs
),

base_tx AS (
  SELECT
    t.block_date,
    t.transaction_hash,
    t.actual_fee_amount / 1e18 AS fee,
    -- decide which match takes priority
    CASE
      -- execute_from_outside_v3 only call
      WHEN CARDINALITY(t.calldata) > 23
        AND contains(cs.addrs, t.calldata[11]) THEN t.calldata[2]
      -- account for VRF preceding execute_from_outside_v3 call
      WHEN CARDINALITY(t.calldata) > 23
        AND contains(cs.addrs, t.calldata[23]) THEN t.calldata[11]
      ELSE NULL
    END AS matched_user
  FROM starknet.transactions t
  CROSS JOIN contract_set cs
  WHERE
    t.block_time >= TIMESTAMP '{start_time}'
    {end_time_constraint}
    AND CARDINALITY(t.calldata) > 23
    AND (
      contains(cs.addrs, t.calldata[11])
      OR contains(cs.addrs, t.calldata[23])
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

-- First time each wallet appears (aligned to day/week/month)
first_seen AS (
  SELECT
    b.matched_user,
    MIN(b.block_date)                                   AS first_ts,
    DATE_TRUNC('day',   MIN(b.block_date))              AS first_day,
    DATE_TRUNC('week',  MIN(b.block_date))              AS first_week,
    DATE_TRUNC('month', MIN(b.block_date))              AS first_month
  FROM base_tx b
  WHERE b.matched_user IS NOT NULL
  GROUP BY 1
),

-- Distinct user/day pairs to power rolling WAU/MAU
daily_user_presence AS (
  SELECT DISTINCT
    DATE_TRUNC('day', b.block_date) AS day,
    b.matched_user
  FROM base_tx b
  WHERE b.matched_user IS NOT NULL
),

-- Activity days (switch to a generated calendar if you want dense dates)
days AS (
  SELECT DISTINCT DATE_TRUNC('day', block_date) AS day
  FROM base_tx
),

-- Rolling 7-day (inclusive) unique users
wau_by_day AS (
  SELECT
    d.day,
    COUNT(DISTINCT dup.matched_user) AS wau
  FROM days d
  LEFT JOIN daily_user_presence dup
    ON dup.day BETWEEN d.day - INTERVAL '6' DAY AND d.day
  GROUP BY 1
),

-- Rolling 30-day (inclusive) unique users
mau_by_day AS (
  SELECT
    d.day,
    COUNT(DISTINCT dup.matched_user) AS mau
  FROM days d
  LEFT JOIN daily_user_presence dup
    ON dup.day BETWEEN d.day - INTERVAL '29' DAY AND d.day
  GROUP BY 1
),

-- Daily KPIs
daily_stats AS (
  SELECT
    DATE_TRUNC('day', b.block_date) AS day,
    COUNT(DISTINCT b.transaction_hash) AS daily_transactions,
    COUNT(DISTINCT b.matched_user)     AS daily_users,
    COUNT(DISTINCT CASE
      WHEN fs.first_day = DATE_TRUNC('day', b.block_date) THEN b.matched_user
    END) AS daily_new_users,
    SUM(b.fee)                        AS daily_fees_strk,
    SUM(b.fee * p.price)              AS daily_fees_usd
  FROM base_tx b
  JOIN prices p
    ON DATE_TRUNC('day', b.block_date) = p.time
  LEFT JOIN first_seen fs
    ON fs.matched_user = b.matched_user
  GROUP BY 1
),

-- Weekly aggregates (distinct users per ISO week)
weekly_stats AS (
  SELECT
    DATE_TRUNC('week', b.block_date)  AS week,
    COUNT(DISTINCT b.transaction_hash) AS weekly_transactions,
    COUNT(DISTINCT b.matched_user)     AS weekly_users,
    COUNT(DISTINCT CASE
      WHEN fs.first_week = DATE_TRUNC('week', b.block_date) THEN b.matched_user
    END) AS weekly_new_users,
    SUM(b.fee)                        AS weekly_fees_strk,
    SUM(b.fee * p.price)              AS weekly_fees_usd
  FROM base_tx b
  JOIN prices p
    ON DATE_TRUNC('day', b.block_date) = p.time
  LEFT JOIN first_seen fs
    ON fs.matched_user = b.matched_user
  GROUP BY 1
),

-- Monthly aggregates (distinct users per month)
monthly_stats AS (
  SELECT
    DATE_TRUNC('month', b.block_date)  AS month,
    COUNT(DISTINCT b.transaction_hash) AS monthly_transactions,
    COUNT(DISTINCT b.matched_user)     AS monthly_users,
    COUNT(DISTINCT CASE
      WHEN fs.first_month = DATE_TRUNC('month', b.block_date) THEN b.matched_user
    END) AS monthly_new_users,
    SUM(b.fee)                        AS monthly_fees_strk,
    SUM(b.fee * p.price)              AS monthly_fees_usd
  FROM base_tx b
  JOIN prices p
    ON DATE_TRUNC('day', b.block_date) = p.time
  LEFT JOIN first_seen fs
    ON fs.matched_user = b.matched_user
  GROUP BY 1
),

-- Overall totals (from base_tx) without any nested SELECT
overall_from_base AS (
  SELECT
    COUNT(DISTINCT b.transaction_hash)             AS overall_transactions,
    COUNT(DISTINCT b.matched_user)                 AS overall_unique_users,
    SUM(b.fee)                                     AS overall_fees_strk,
    SUM(b.fee * p.price)                           AS overall_fees_usd
  FROM base_tx b
  JOIN prices p
    ON DATE_TRUNC('day', b.block_date) = p.time
),

-- Overall ratio using the daily aggregates (avoids double-counting by day)
overall_from_daily AS (
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
LEFT JOIN wau_by_day   w7 ON w7.day  = d.day
LEFT JOIN mau_by_day   m30 ON m30.day = d.day
LEFT JOIN weekly_stats w  ON w.week  = DATE_TRUNC('week',  d.day)
LEFT JOIN monthly_stats m ON m.month = DATE_TRUNC('month', d.day)
CROSS JOIN overall_from_base  ob
CROSS JOIN overall_from_daily ofd
ORDER BY d.day;