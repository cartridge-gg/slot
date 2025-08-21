# Merkle Drop Commands

The Slot CLI provides commands to create and manage merkle drops for token distribution campaigns.

## Overview

Merkle drops are an efficient way to distribute tokens to a large number of recipients while minimizing gas costs. The system uses a merkle tree to prove eligibility for claiming tokens without storing all recipient data onchain.

The merkle root is automatically calculated server-side from the provided claims data, ensuring consistency and eliminating the need for manual merkle tree generation.

## Commands

### Build Merkle Tree

Build a merkle tree by querying token holders from on-chain NFT contract(s) via RPC. Supports both single-contract and multi-contract modes, with optional precalculated multi-token rewards.

**Aliases:** `slot md b`

```bash
slot merkle-drops build [OPTIONS]
```

#### Required Parameters

- `--name <NAME>` - Name for the merkle drop
- `--rpc-url <RPC_URL>` - Network RPC URL (e.g., https://ethereum-rpc.publicnode.com)
- `--description <DESCRIPTION>` - Description of the merkle drop
- `--claim-contract <CLAIM_CONTRACT>` - Claim contract address for the merkle drop
- `--entrypoint <ENTRYPOINT>` - Entrypoint address for claiming
- `--block-height <BLOCK_HEIGHT>` - Block height to query at (required for deterministic snapshots)

#### Contract Selection (Choose One)

**Single Contract Mode:**
- `--contract-address <CONTRACT_ADDRESS>` - Single NFT contract address to query
- `--from-id <FROM_ID>` - Starting token ID (default: 1)
- `--to-id <TO_ID>` - Ending token ID (default: 8000)

**Multi-Contract Mode:**
- `--contracts-config <FILE>` - JSON file with contract configurations

#### Reward Calculation (Optional)

**On-Chain Mode (Default):**
- Uses token ownership as-is from blockchain

**Precalculated Rewards Mode:**
- `--use-precalculated` - Enable precalculated multi-token rewards
- `--rewards-config <FILE>` - JSON file with reward amounts per contract

#### Optional Parameters

- `--network <NETWORK>` - Network name (e.g., ETH, BASE) (default: ETH)
- `--output <OUTPUT>` - Output file path (default: merkle_drop.json)
- `--delay-ms <DELAY_MS>` - Delay between RPC calls in milliseconds (default: 10)
- `--concurrency <CONCURRENCY>` - Number of concurrent RPC requests (default: 10)

#### Configuration Files

**Multi-Contract Configuration** (`contracts_config.json`):
```json
[
  {
    "address": "0x8707276DF042E89669d69A177d3DA7dC78bd8723",
    "from_id": 1,
    "to_id": 8000
  },
  {
    "address": "0xabcdef1234567890abcdef1234567890abcdef12",
    "from_id": 1,
    "to_id": 5000
  }
]
```

**Multi-Token Rewards Configuration** (`rewards_config.json`):
```json
{
  "contracts": {
    "0x8707276DF042E89669d69A177d3DA7dC78bd8723": [100, 25],
    "0xabcdef1234567890abcdef1234567890abcdef12": [50, 10]
  }
}
```
*Each array represents rewards per NFT: [Token A amount, Token B amount, ...]*

#### Examples

**Single Contract Mode (Traditional):**
```bash
slot merkle-drops build \
  --name "Dope Collection" \
  --contract-address "0x8707276DF042E89669d69A177d3DA7dC78bd8723" \
  --rpc-url "https://ethereum-rpc.publicnode.com" \
  --network "ETH" \
  --description "Dope owners can claim their rewards" \
  --claim-contract "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589" \
  --entrypoint "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589" \
  --block-height 22728943 \
  --from-id 1 \
  --to-id 8000 \
  --concurrency 20 \
  --output dope_loot_snapshot.json
```

**Multi-Contract On-Chain Mode:**
```bash
slot merkle-drops build \
  --name "Multi Collection Drop" \
  --contracts-config contracts_config.json \
  --rpc-url "https://ethereum-rpc.publicnode.com" \
  --network "ETH" \
  --description "Multiple collection holders get rewards" \
  --claim-contract "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589" \
  --entrypoint "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589" \
  --block-height 22728943 \
  --output multi_collection_snapshot.json
```

**Multi-Contract + Multi-Token Rewards:**
```bash
slot merkle-drops build \
  --name "Dual Token Rewards" \
  --contracts-config contracts_config.json \
  --use-precalculated \
  --rewards-config rewards_config.json \
  --rpc-url "https://ethereum-rpc.publicnode.com" \
  --network "ETH" \
  --description "Token A and Token B rewards based on NFT holdings" \
  --claim-contract "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589" \
  --entrypoint "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589" \
  --block-height 22728943 \
  --output dual_token_rewards.json
```

#### Build Process & Reward Calculation

The build command supports two distinct approaches for determining token distributions:

**On-Chain Calculation Mode (Default):**
- **Use Case**: When smart contracts will calculate rewards based on NFT ownership
- **Process**: 
  1. Query token IDs in parallel across all configured contracts
  2. Build a map of owner addresses to token IDs they hold
  3. Generate merkle tree with NFT ownership data
  4. Smart contract calculates final reward amounts during claiming
- **Output**: Snapshot contains the actual NFT token IDs owned by each address
- **Example**: `["0xAddress1", [1, 2, 3]]` means address owns NFT tokens 1, 2, and 3

**Precalculated Rewards Mode:**
- **Use Case**: When you want to determine exact reward amounts off-chain instead of on-chain
- **Why Use This**: 
  - Avoid complex on-chain calculation logic and gas costs
  - Implement sophisticated reward formulas (e.g., rarity-based rewards, time-based multipliers)
  - Support multi-token distributions with different rates per collection
- **Process**:
  1. Query NFT holders from all configured contracts
  2. Calculate exact reward amounts per holder based on:
     - Number of NFTs owned in each contract
     - Reward rate configuration per contract per token type
  3. Generate merkle tree with final calculated reward amounts
  4. Smart contract simply distributes the precalculated amounts
- **Output**: Snapshot contains calculated token amounts, not NFT IDs
- **Example**: `["0xAddress1", [400, 95]]` means address receives 400 Token A + 95 Token B

**Key Difference:**
- **On-Chain Mode**: NFT ownership → Smart contract calculates rewards → Distribution
- **Precalculated Mode**: NFT ownership → Build tool calculates rewards → Smart contract distributes predetermined amounts

#### Practical Example

**Scenario**: You want to reward holders of two NFT collections with different token amounts:
- Premium Collection (0xAAA): 100 Token A + 25 Token B per NFT
- Standard Collection (0xBBB): 50 Token A + 10 Token B per NFT

**On-Chain Approach:**
```bash
# Build with NFT ownership data
slot merkle-drops build \
  --contracts-config contracts.json \
  --name "NFT Rewards" \
  # ... other params

# Output: ["0xHolder1", [1, 2, 5, 101, 102]]  (NFT IDs from both collections)
# Smart contract must:
# - Determine which NFTs are from which collection  
# - Apply different reward rates per collection
# - Calculate: 3 premium NFTs × [100,25] + 2 standard NFTs × [50,10] = [400,95]
```

**Precalculated Approach:**
```bash
# Build with precalculated rewards
slot merkle-drops build \
  --contracts-config contracts.json \
  --use-precalculated \
  --rewards-config rewards.json \
  --name "NFT Rewards" \
  # ... other params

# Output: ["0xHolder1", [400, 95]]  (Final calculated amounts)
# Smart contract simply distributes 400 Token A + 95 Token B
```

**Benefits of On-Chain Calculation Mode:**
- ✅ **Full Transparency**: Reward calculation logic is public and verifiable on-chain
- ✅ **Trustless**: No need to trust off-chain calculations - anyone can verify the math
- ✅ **Immutable Logic**: Reward formulas are permanently encoded in smart contracts
- ✅ **Real-time Verification**: Community can audit and verify distribution fairness
- ✅ **Decentralized**: No reliance on external tools or processes for reward calculation

**Benefits of Precalculated Mode:**
- ✅ **Simpler Smart Contracts**: No complex calculation logic needed on-chain
- ✅ **Lower Gas Costs**: No on-chain computation during claims, just token transfers
- ✅ **Complex Formulas**: Support sophisticated reward logic (rarity, time-based bonuses, etc.)
- ✅ **Multi-Token Support**: Easy distribution of multiple token types simultaneously
- ✅ **Performance**: Handle complex calculations without gas limit concerns
- ✅ **Flexibility**: Implement reward logic that would be too expensive on-chain

**Trade-offs:**
- **On-Chain**: Higher gas costs but maximum trustlessness and transparency
- **Precalculated**: Lower gas costs but requires trust in the build tool's calculations

#### Output Formats

**Single Token Output:**
```json
{
  "name": "Dope Collection",
  "network": "ETH",
  "description": "Dope owners can claim their rewards",
  "claim_contract": "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589",
  "entrypoint": "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589",
  "merkle_root": "0x8f7c9e2b1a5d4e8f3c6b9a2d7e1f4c8b5e9a3d7c1f8e4b2a6d9c3f7e1a5b8d2c6f",
  "snapshot": [
    ["0xAddress1", [1, 2, 3]],        // Token IDs owned
    ["0xAddress2", [4, 5, 6]]
  ]
}
```

**Multi-Token Rewards Output:**
```json
{
  "name": "Dual Token Rewards",
  "network": "ETH", 
  "description": "Token A and Token B rewards based on NFT holdings",
  "claim_contract": "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589",
  "entrypoint": "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589",
  "merkle_root": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "snapshot": [
    ["0xAddress1", [400, 95]],        // 400 Token A + 95 Token B
    ["0xAddress2", [300, 75]]         // 300 Token A + 75 Token B
  ]
}
```

**Multi-Contract Aggregation:**
When using multiple contracts, token holdings are automatically aggregated per address across all contracts before reward calculation.

#### Compatibility

The output file can be directly used with `slot merkle-drops create json --file <output>` - the create command automatically handles both single-token and multi-token formats.

### Create Merkle Drop

Create a new merkle drop with specified recipients and token allocations. The CLI supports three different creation methods:

1. **From Parameters** - Specify all configuration parameters individually
2. **From JSON** - Load configuration and data from a JSON file
3. **From Preset** - Use a predefined preset from the community presets repository

**Aliases:** `slot md c`

#### Method 1: From Parameters

Create a merkle drop by specifying all parameters individually.

```bash
slot merkle-drops create params [OPTIONS]
```

##### Required Parameters

- `--name <NAME>` - Unique name for the merkle drop
- `--network <NETWORK>` - Network (e.g., ETH, STARKNET)
- `--contract <CONTRACT>` - Contract address
- `--entrypoint <ENTRYPOINT>` - Entrypoint address
- `--data-file <DATA_FILE>` - Path to JSON file containing merkle drop data

##### Optional Parameters

- `--description <DESCRIPTION>` - Description of the merkle drop
- `--args <ARGS>` - Arguments for the contract call (comma-separated)

#### Method 2: From JSON

Create a merkle drop from a complete JSON configuration file.

```bash
slot merkle-drops create json --file <CONFIG_FILE> --team <TEAM>
```

##### Required Parameters

- `--file <CONFIG_FILE>` - Path to JSON configuration file
- `--team <TEAM>` - Team name to associate the merkle drop with

#### Method 3: From Preset

Create a merkle drop using a community preset configuration.

```bash
slot merkle-drops create preset --project <PROJECT> --name <NAME> [--network <NETWORK>]
```

##### Required Parameters

- `--project <PROJECT>` - Project/preset name from [cartridge-gg/presets](https://github.com/cartridge-gg/presets/tree/main/configs)
- `--name <NAME>` - Merkle drop name from the preset

##### Optional Parameters

- `--network <NETWORK>` - Network to use from preset (default: SN_MAIN)

## Data File Formats

### Parameters Method - Data File Format

For the `params` method, the data file must be a JSON array where each entry contains:
1. Recipient address (string) 
2. Token amounts or IDs (array of integers)

**Note:** Supports both single-token and multi-token formats from build command output.

**Single Token Data:**
```json
[
  [
    "0xD6E9625d91dc1F2823EF60Eb902266f7dd9D75Df",
    [1, 5352, 5533, 7443]  // Token IDs owned
  ],
  [
    "0x1234567890123456789012345678901234567890", 
    [42]  // Single token amount
  ]
]
```

**Multi-Token Rewards Data:**
```json
[
  [
    "0xD6E9625d91dc1F2823EF60Eb902266f7dd9D75Df",
    [400, 95]  // 400 Token A + 95 Token B
  ],
  [
    "0x1234567890123456789012345678901234567890",
    [300, 75, 15]  // Token A + Token B + Token C amounts
  ]
]
```

### JSON Method - Configuration File Format

For the `json` method, the configuration file must contain both the merkle drop configuration and the recipient data:

```json
{
  "name": "my-drop-2024",
  "config": {
    "description": "Community rewards for active users",
    "network": "SN_MAIN",
    "contract": "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589",
    "entrypoint": "distribute",
    "args": ["TOKEN_ID", "MERKLE_PROOF"]
  },
  "data": [
    [
      "0xD6E9625d91dc1F2823EF60Eb902266f7dd9D75Df",
      [400, 95]  // Multi-token rewards: Token A + Token B amounts
    ],
    [
      "0x1234567890123456789012345678901234567890", 
      [300, 75]  // Different reward amounts per holder
    ]
  ]
}
```

### Preset Method - Community Presets

Presets are managed in the [cartridge-gg/presets](https://github.com/cartridge-gg/presets) repository. Each preset contains:
- Configuration in `config.json`
- Merkle drop data in `merkledrops/<key>.json`

Available presets include:
- `dope-wars` - Dope Wars NFT drops
- And more community-maintained presets

## Examples

### Method 1: From Parameters

#### Basic Merkle Drop Creation

```bash
slot merkle-drops create params \
  --name "dope-drop-2024-q1" \
  --description "Dope owners can claim their rewards" \
  --network "ETH" \
  --contract "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589" \
  --entrypoint "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589" \
  --args "TOKEN_ID,MERKLE_PROOF" \
  --data-file ./recipients.json
```

#### Using Aliases

```bash
slot md c params \
  --name "rewards-2024" \
  --network "STARKNET" \
  --contract "0x123..." \
  --entrypoint "0x456..." \
  --data-file ./community_rewards.json
```

#### Minimal Example (No Optional Args)

```bash
slot merkle-drops create params \
  --name "simple-001" \
  --network "ETH" \
  --contract "0x123..." \
  --entrypoint "0x456..." \
  --data-file ./simple_drop.json
```

### Method 2: From JSON Configuration

#### Complete Configuration in JSON

```bash
slot merkle-drops create json \
  --file ./complete-drop-config.json \
  --team "my-team"
```

Where `complete-drop-config.json` contains:
```json
{
  "key": "community-rewards-q4",
  "config": {
    "description": "Q4 community rewards distribution",
    "network": "SN_MAIN",
    "contract": "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589",
    "entrypoint": "claim_rewards",
    "args": ["RECIPIENT", "AMOUNT", "PROOF"]
  },
  "data": [
    ["0x1234...", [400, 95]],   // Multi-token: 400 Token A + 95 Token B
    ["0x5678...", [300, 75]]    // 300 Token A + 75 Token B
  ]
}
```

### Method 3: From Community Presets

#### Using Dope Wars Preset

```bash
slot merkle-drops create preset \
  --project "dope-wars" \
  --name "dope" \
  --network "SN_MAIN"
```

#### Using Custom Preset

```bash
slot merkle-drops create preset \
  --project "my-community-preset" \
  --name "season-1-rewards"
```

#### Preset with Different Network

```bash
slot merkle-drops create preset \
  --project "dope-wars" \
  --name "dope" \
  --network "ETH"
```

## Output

Upon successful creation, the command displays:

```
✅ Merkle Drop Created Successfully
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🏢 Details:
  • ID: merkle_drop_12345
  • Name: dope-drop-2024-q1
  • Description: Dope owners can claim their rewards

🔗 Contract Details:
  • Network: ETH
  • Contract: 0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589
  • Entrypoint: 0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589
  • Args: ["TOKEN_ID", "MERKLE_PROOF"]

🌳 Merkle Details:
  • Root: 0x8f7c9e2b1a5d4e8f3c6b9a2d7e1f4c8b5e9a3d7c1f8e4b2a6d9c3f7e1a5b8d2c6f (auto-generated)
  • Entries: 3
  • Created: 2024-08-15T10:30:00Z
```

## Data Validation

The command performs comprehensive validation:

- ✅ **JSON Format**: Ensures data file is valid JSON
- ✅ **Array Structure**: Validates top-level array format
- ✅ **Entry Format**: Each entry must have exactly 2 elements
- ✅ **Address Format**: First element must be a string (address)
- ✅ **Token Data**: Second element must be an array of integers (supports both token IDs and multi-token amounts)
- ✅ **Multi-Token Support**: Handles both numeric and string array elements from build command output

## Error Handling

Common error scenarios:

### Invalid Data Format
```bash
Entry 0 must have exactly 2 elements: [address, token_ids]
```

### Missing Required Parameters
```bash
error: the following required arguments were not provided:
  --name <NAME>
  --network <NETWORK>
```

### Preset Not Found
```bash
Preset 'invalid-preset' not found. Check available presets at https://github.com/cartridge-gg/presets/tree/main/configs
```

### Invalid JSON Configuration
```bash
Failed to parse JSON configuration: missing field `key`
```

### API Errors
```bash
API error: 422 Unprocessable Entity
```

## Authentication

Merkle drop operations require authentication. Ensure you're logged in:

```bash
slot auth login
```

## Discovering Available Presets

To find available community presets:

1. Browse the [presets repository](https://github.com/cartridge-gg/presets/tree/main/configs)
2. Each folder represents a preset (e.g., `dope-wars`)
3. Check `config.json` for available merkle drops under the `merkledrops` section
4. Use the merkle drop name from the configuration

### Preset Structure Example

Preset `dope-wars` contains:
```
configs/dope-wars/
├── config.json                 # Main preset configuration
└── merkledrops/
    └── dope.json              # Merkle drop data for name "dope"
```

## Best Practices

1. **Unique Names**: Always use unique names for each merkle drop to avoid conflicts
2. **Method Selection**: 
   - Use `params` for one-off drops with custom configuration
   - Use `json` for complex drops with version control needs
   - Use `preset` for community-standard drops
3. **Data Validation**: Validate recipient data before creation to avoid errors
4. **Backup Data**: Keep backups of your merkle drop data files
5. **Test First**: Test with small datasets before large-scale deployments
6. **Preset Updates**: When using presets, check for updates in the community repository
7. **Multi-Contract Strategy**: Use multi-contract mode for ecosystem-wide drops across multiple collections
8. **Multi-Token Planning**: Design reward tokenomics carefully when using multi-token rewards
9. **Block Height**: Always specify block height for reproducible snapshots across multiple builds
10. **Reward Validation**: Verify reward calculations match expected tokenomics before deployment

## Related Commands

- `slot auth login` - Authenticate with Slot API

For more information, see the [Slot CLI documentation](https://docs.cartridge.gg/slot).
