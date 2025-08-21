# Merkle Drop Commands

The Slot CLI provides commands to create and manage merkle drops for token distribution campaigns using a modular workflow.

## Overview

Merkle drops are an efficient way to distribute tokens to a large number of recipients while minimizing gas costs. The system uses a merkle tree to prove eligibility for claiming tokens without storing all recipient data onchain.

The Slot CLI uses a three-step modular approach:
1. **Snapshot** - Collect NFT holder data from individual contracts
2. **Process** - Calculate rewards across multiple snapshots  
3. **Create** - Deploy via API (server handles merkle tree generation)

## Commands

### 1. Create Snapshot

Create a snapshot of token holders from a single ERC721 contract at a specific block height.

**Aliases:** `slot md s`

```bash
slot merkle-drops snapshot [OPTIONS]
```

#### Required Parameters

- `--contract-address <CONTRACT_ADDRESS>` - ERC721 contract address to query
- `--rpc-url <RPC_URL>` - Network RPC URL (e.g., https://ethereum-rpc.publicnode.com)
- `--block-height <BLOCK_HEIGHT>` - Block height for deterministic snapshots

#### Optional Parameters

- `--network <NETWORK>` - Network name (e.g., ETH, BASE) (default: ETH)
- `--from-id <FROM_ID>` - Starting token ID (default: 1)
- `--to-id <TO_ID>` - Ending token ID (default: 8000)
- `--output <OUTPUT>` - Output file path (default: snapshot.json)
- `--delay-ms <DELAY_MS>` - Delay between RPC calls in milliseconds (default: 10)
- `--concurrency <CONCURRENCY>` - Number of concurrent RPC requests (default: 10)

#### Examples

**Basic Snapshot:**
```bash
slot merkle-drops snapshot \
  --contract-address "0x8707276DF042E89669d69A177d3DA7dC78bd8723" \
  --rpc-url "https://ethereum-rpc.publicnode.com" \
  --network "ETH" \
  --block-height 18500000 \
  --from-id 1 \
  --to-id 8000 \
  --output "dope_eth_snapshot.json"
```

**Multiple Contract Snapshots:**
```bash
# Create separate snapshots for each contract
slot merkle-drops snapshot \
  --contract-address "0x8707276DF042E89669d69A177d3DA7dC78bd8723" \
  --rpc-url "https://ethereum-rpc.publicnode.com" \
  --network "ETH" \
  --block-height 18500000 \
  --output "dope_snapshot.json"

slot merkle-drops snapshot \
  --contract-address "0xBC4CA0EdA7647A8aB7C2061c2E118A18a936f13D" \
  --rpc-url "https://ethereum-rpc.publicnode.com" \
  --network "ETH" \
  --block-height 18500000 \
  --output "apes_snapshot.json"
```

#### Output Format

```json
{
  "contract_address": "0x8707276DF042E89669d69A177d3DA7dC78bd8723",
  "network": "ETH",
  "block_height": 18500000,
  "snapshot": {
    "0xAddress1": [1, 5, 23],
    "0xAddress2": [42, 100, 150]
  }
}
```

### 2. Process Rewards

Calculate multi-token rewards from multiple snapshot files using a rewards configuration.

**Aliases:** `slot md p`

```bash
slot merkle-drops process [OPTIONS]
```

#### Required Parameters

- `--name <NAME>` - Name for the merkle drop
- `--description <DESCRIPTION>` - Description of the merkle drop
- `--claim-contract <CLAIM_CONTRACT>` - Claim contract address for token distribution
- `--entrypoint <ENTRYPOINT>` - Entrypoint address for claiming
- `--snapshots <SNAPSHOTS>` - Comma-separated list of snapshot files to process
- `--rewards-config <REWARDS_CONFIG>` - JSON file with reward amounts per contract

#### Optional Parameters

- `--output <OUTPUT>` - Output file path (default: processed_rewards.json)

#### Configuration Files

**Rewards Configuration** (`rewards_config.json`):
```json
{
  "contracts": {
    "0x8707276DF042E89669d69A177d3DA7dC78bd8723": [1000, 500],
    "0xBC4CA0EdA7647A8aB7C2061c2E118A18a936f13D": [750, 250]
  }
}
```
*Each array represents rewards per NFT: [Token A amount, Token B amount, ...]*

#### Examples

**Dual Token Rewards:**
```bash
slot merkle-drops process \
  --name "Dual Token Campaign" \
  --description "Token A and Token B rewards for NFT ecosystem" \
  --snapshots "dope_snapshot.json,apes_snapshot.json" \
  --rewards-config "rewards_config.json" \
  --claim-contract "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589" \
  --entrypoint "claim_dual_rewards" \
  --output "dual_token_rewards.json"
```

**Single Token Rewards:**
```bash
slot merkle-drops process \
  --name "Community Rewards" \
  --description "Single token rewards for holders" \
  --snapshots "dope_snapshot.json" \
  --rewards-config "single_token_rewards.json" \
  --claim-contract "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589" \
  --entrypoint "claim_rewards" \
  --output "community_rewards.json"
```

#### Process Logic

**Reward Calculation:**
1. Load all snapshot files and validate network consistency
2. Load rewards configuration and validate token count consistency
3. For each holder across all snapshots:
   - Count NFTs held in each contract
   - Multiply by per-contract reward rates for each token type
   - Sum rewards across all contracts

**Example Calculation:**
- Holder owns 3 NFTs from Contract A (1000 Token A + 500 Token B per NFT)
- Holder owns 2 NFTs from Contract B (750 Token A + 250 Token B per NFT)
- **Total**: 3×[1000,500] + 2×[750,250] = **[4500, 2000]**

#### Output Format

```json
{
  "name": "Dual Token Campaign",
  "network": "ETH",
  "description": "Token A and Token B rewards for NFT ecosystem",
  "claim_contract": "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589",
  "entrypoint": "claim_dual_rewards",
  "snapshot": [
    ["0xAddress1", [4500, 2000]],
    ["0xAddress2", [3250, 1500]]
  ]
}
```

### 3. Create Merkle Drop

Create a new merkle drop by submitting processed data to the Slot API. The server handles all merkle tree generation and proof creation.

**Aliases:** `slot md c`

#### Method 1: From Processed Data (Recommended)

Create a merkle drop from processed reward data.

```bash
slot merkle-drops create json --file <PROCESSED_FILE>
```

##### Required Parameters

- `--file <PROCESSED_FILE>` - Path to JSON file from `slot merkle-drops process`

##### Example

```bash
slot merkle-drops create json --file "dual_token_rewards.json"
```

#### Method 2: From Parameters

Create a merkle drop by specifying parameters individually with a separate data file.

```bash
slot merkle-drops create params [OPTIONS]
```

##### Required Parameters

- `--name <NAME>` - Unique name for the merkle drop
- `--network <NETWORK>` - Network (e.g., ETH, STARKNET)
- `--contract <CONTRACT>` - Claim contract address
- `--entrypoint <ENTRYPOINT>` - Entrypoint address
- `--data-file <DATA_FILE>` - Path to JSON file containing reward data

##### Optional Parameters

- `--description <DESCRIPTION>` - Description of the merkle drop

#### Method 3: From Preset

Create a merkle drop using community preset configurations.

```bash
slot merkle-drops create preset --project <PROJECT> --name <NAME> [--network <NETWORK>]
```

##### Required Parameters

- `--project <PROJECT>` - Project/preset name from [cartridge-gg/presets](https://github.com/cartridge-gg/presets/tree/main/configs)
- `--name <NAME>` - Merkle drop name from the preset

##### Optional Parameters

- `--network <NETWORK>` - Network to use from preset (default: SN_MAIN)

## Complete Workflow Examples

### Single Contract Campaign

```bash
# 1. Create snapshot
slot merkle-drops snapshot \
  --contract-address "0x8707276DF042E89669d69A177d3DA7dC78bd8723" \
  --rpc-url "https://ethereum-rpc.publicnode.com" \
  --network "ETH" \
  --block-height 18500000 \
  --output "dope_snapshot.json"

# 2. Process with single token rewards  
slot merkle-drops process \
  --name "Dope Rewards" \
  --description "Rewards for Dope NFT holders" \
  --snapshots "dope_snapshot.json" \
  --rewards-config "single_rewards.json" \
  --claim-contract "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589" \
  --entrypoint "claim_dope" \
  --output "dope_rewards.json"

# 3. Create merkle drop
slot merkle-drops create json --file "dope_rewards.json"
```

### Multi-Contract + Multi-Token Campaign

```bash
# 1. Create snapshots for each contract
slot merkle-drops snapshot \
  --contract-address "0x8707276DF042E89669d69A177d3DA7dC78bd8723" \
  --rpc-url "https://ethereum-rpc.publicnode.com" \
  --network "ETH" \
  --block-height 18500000 \
  --output "dope_snapshot.json"

slot merkle-drops snapshot \
  --contract-address "0xBC4CA0EdA7647A8aB7C2061c2E118A18a936f13D" \
  --rpc-url "https://ethereum-rpc.publicnode.com" \
  --network "ETH" \
  --block-height 18500000 \
  --output "apes_snapshot.json"

# 2. Process with multi-token rewards
slot merkle-drops process \
  --name "Ecosystem Rewards" \
  --description "Token A and Token B rewards for NFT ecosystem" \
  --snapshots "dope_snapshot.json,apes_snapshot.json" \
  --rewards-config "dual_token_rewards.json" \
  --claim-contract "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589" \
  --entrypoint "claim_ecosystem" \
  --output "ecosystem_rewards.json"

# 3. Create merkle drop
slot merkle-drops create json --file "ecosystem_rewards.json"
```

## Data File Formats

### Parameters Method - Data File Format

For the `params` method, the data file must be a JSON array where each entry contains:
1. Recipient address (string)
2. Token amounts (array of integers)

```json
[
  [
    "0xD6E9625d91dc1F2823EF60Eb902266f7dd9D75Df",
    [400, 95]
  ],
  [
    "0x1234567890123456789012345678901234567890",
    [300, 75, 15]
  ]
]
```

### Preset Method - Community Presets

Presets are managed in the [cartridge-gg/presets](https://github.com/cartridge-gg/presets) repository. Each preset contains:
- Configuration in `config.json`
- Merkle drop data in `merkledrops/<key>.json`

Available presets include:
- `dope-wars` - Dope Wars NFT drops
- And more community-maintained presets

## Benefits of Modular Approach

### Snapshot Benefits
- ✅ **Caching**: Reuse expensive RPC queries across multiple campaigns
- ✅ **Auditability**: Clear record of exact NFT ownership at specific block heights
- ✅ **Efficiency**: Generate once, use multiple times with different reward configurations

### Process Benefits  
- ✅ **Flexibility**: Mix and match snapshots from different contracts and networks
- ✅ **Multi-Token Support**: Handle complex tokenomics with multiple reward tokens
- ✅ **Transparency**: Clear visibility into total token distribution amounts
- ✅ **Validation**: Comprehensive error checking before deployment

### Create Benefits
- ✅ **Server-Side Optimization**: API handles merkle tree generation and proof creation
- ✅ **Simplified Deployment**: No local cryptographic computation required
- ✅ **Reduced Complexity**: Focus on data submission rather than merkle tree management

## Output

Upon successful creation, the command displays:

```
✅ Merkle Drop Created Successfully
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🏢 Details:
  • ID: merkle_drop_12345
  • Name: Ecosystem Rewards
  • Description: Token A and Token B rewards for NFT ecosystem

🔗 Contract Details:
  • Network: ETH
  • Contract: 0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589
  • Entrypoint: claim_ecosystem

🌳 Merkle Details:
  • Root: 0x8f7c9e2b1a5d4e8f3c6b9a2d7e1f4c8b5e9a3d7c1f8e4b2a6d9c3f7e1a5b8d2c6f (auto-generated)
  • Entries: 150
  • Created: 2024-08-21T10:30:00Z
```

## Data Validation

The commands perform comprehensive validation:

- ✅ **Network Consistency**: All snapshots must be from the same network
- ✅ **Token Count Consistency**: All contracts must have the same number of reward tokens
- ✅ **JSON Format**: Ensures all data files are valid JSON
- ✅ **Address Format**: Validates recipient addresses
- ✅ **Reward Data**: Validates reward amounts and token arrays

## Error Handling

Common error scenarios:

### Snapshot Errors
```bash
Error querying token 1234: contract call reverted
Warning: Error querying token 5678: timeout
```

### Process Errors  
```bash
Network mismatch: snapshot for contract 0xAAA is on network 'ETH', expected 'BASE'
All contracts must have the same number of reward tokens. Contract 0xBBB has 3 tokens, expected 2
```

### Create Errors
```bash
Failed to parse JSON file. Expected output from 'slot merkle-drops process'
API error: 422 Unprocessable Entity
```

## Authentication

Merkle drop operations require authentication. Ensure you're logged in:

```bash
slot auth login
```

## Best Practices

1. **Snapshot Strategy**: 
   - Use consistent block heights across related snapshots
   - Name snapshot files clearly (e.g., `dope_eth_18500000.json`)
   - Store snapshots for reuse across multiple campaigns

2. **Process Planning**:
   - Design reward tokenomics carefully before processing
   - Validate total distribution amounts match available token supply
   - Test with small datasets before large-scale processing

3. **Multi-Token Design**:
   - Ensure all contracts use the same number of reward tokens
   - Consider token economic implications of different reward rates
   - Plan for appropriate reward token liquidity

4. **Network Consistency**:
   - Use the same network and block height for related snapshots
   - Verify RPC endpoint reliability for large token ranges
   - Consider rate limiting and concurrency settings

5. **Data Management**:
   - Keep backups of snapshot and processed data files
   - Version control reward configurations
   - Document reward calculation rationale

## Use Cases

### Ecosystem-Wide Campaigns
Create rewards for holders across multiple NFT collections:
- Take snapshots of all ecosystem contracts
- Define tiered rewards based on collection rarity/value
- Process combined rewards for unified campaign

### Tiered Tokenomics
Implement sophisticated reward structures:
- Premium collections: Higher reward rates
- Standard collections: Base reward rates  
- Multi-token distributions: Different tokens for different purposes

### Cached Data Reuse
Leverage snapshot caching for efficiency:
- Generate snapshots once for expensive RPC operations
- Reuse snapshots across multiple reward campaigns
- Test different reward configurations without re-querying blockchain

## Related Commands

- `slot auth login` - Authenticate with Slot API

For more information, see the [Slot CLI documentation](https://docs.cartridge.gg/slot).