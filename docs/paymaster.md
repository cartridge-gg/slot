## Dune Analytics Queries

Generate Dune Analytics queries to analyze your paymaster's transaction data:

```sh
slot paymaster <paymaster-name> dune [OPTIONS]
```

### Example Dashboard
See a live example of paymaster analytics at [Blob Arena Stats](https://dune.com/broody/blobert-arena-stats) on Dune Analytics.

### Basic Usage

Generate a comprehensive SQL query for your paymaster:

```sh
slot paymaster my-game-pm dune
```

The query provides exhaustive analysis including:
- Finds all execute_from_outside_v3 selectors in transaction calldata
- Handles both normal calls and multi-call VRF helpers
- Matches all patterns including nested VRF calls
- Comprehensive metrics with daily, weekly, and monthly breakdowns

By default the generated SQL only includes paymaster-sponsored calls. Pass `--paymaster-only false` to aggregate every transaction that hits your tracked contracts, whether or not the paymaster covered the fee.

### Time Period Options

By default, queries use the paymaster's creation time. You can specify a custom time period:

```sh
# Last 24 hours
slot paymaster my-game-pm dune --last 24hr

# Last week
slot paymaster my-game-pm dune --last 1week
```

**Time Period Options:**
- `1hr`, `2hr`, `24hr`
- `1day`, `2day`, `7day`
- `1week`

### Dune Template Parameters

For dynamic queries in Dune dashboards, use template parameters:

```sh
slot paymaster my-game-pm dune --dune-params
```

This generates a query with `{{start_time}}` and `{{end_time}}` parameters that you can configure in your Dune dashboard.

### Query Output

The command generates a comprehensive SQL query that includes:

**Daily Metrics:**
- Transaction counts and unique users
- New vs returning users
- Fees in STRK and USD
- Transactions per user ratio

**Rolling Windows:**
- 7-day active users (WAU)
- 30-day active users (MAU)

**Aggregated Views:**
- Weekly and monthly summaries
- Overall totals and averages
- User acquisition and retention metrics

**Policy Filtering:**
The query automatically excludes common token contracts (STRK, ETH, LORDS) from analysis to focus on your game-specific transactions.

### Usage Tips

**For Dashboard Creation:**
- Use `--dune-params` for interactive dashboards
- Copy the generated SQL directly into Dune Analytics
- Set up time range parameters for flexible analysis

**For One-time Analysis:**
- Specify `--last` with appropriate time period
- Query will include actual timestamps for immediate execution

:::tip
The query is optimized for comprehensive analysis but may timeout on very long time ranges. For historical analysis spanning months, consider breaking it into smaller time periods or using Dune's incremental refresh features.
:::

### Common Use Cases

**Growth Analysis:**
```sh
slot paymaster my-game-pm dune --last 1week
```
Analyze weekly growth trends, user acquisition, and engagement patterns.

**Daily Monitoring:**
```sh  
slot paymaster my-game-pm dune --last 24hr
```
Monitor recent activity, transaction success rates, and costs.

**Dashboard Setup:**
```sh
slot paymaster my-game-pm dune --dune-params
```
Create flexible dashboards with configurable time ranges. 

**Full Activity Overview:**
```sh
slot paymaster my-game-pm dune --paymaster-only false
```
Measure total on-chain engagement by combining sponsored and self-funded transactions.
