# Merkle Drop Commands

The Slot CLI provides commands to create and manage merkle drops for token distribution campaigns.

## Overview

Merkle drops are an efficient way to distribute tokens to a large number of recipients while minimizing gas costs. The system uses a merkle tree to prove eligibility for claiming tokens without storing all recipient data on-chain.

## Commands

### Create Merkle Drop

Create a new merkle drop with specified recipients and token allocations.

```bash
slot merkle-drops create [OPTIONS]
```

**Aliases:** `slot md c`

#### Required Parameters

- `--name <NAME>` - Name of the merkle drop
- `--project <PROJECT>` - Project to associate the merkle drop with
- `--key <KEY>` - Unique key for the merkle drop
- `--network <NETWORK>` - Network (e.g., ETH, STARKNET)
- `--contract <CONTRACT>` - Contract address
- `--entrypoint <ENTRYPOINT>` - Entrypoint address
- `--data-file <DATA_FILE>` - Path to JSON file containing merkle drop data

#### Optional Parameters

- `--description <DESCRIPTION>` - Description of the merkle drop
- `--args <ARGS>` - Arguments for the contract call (comma-separated)

## Data File Format

The data file must be a JSON array where each entry contains:
1. Recipient address (string)
2. Token IDs or amounts (array of integers)

```json
[
  [
    "0xD6E9625d91dc1F2823EF60Eb902266f7dd9D75Df",
    [1, 5352, 5533, 7443]
  ],
  [
    "0x1234567890123456789012345678901234567890", 
    [100, 200, 300]
  ],
  [
    "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd",
    [42]
  ]
]
```

## Examples

### Basic Merkle Drop Creation

```bash
slot merkle-drops create \
  --name "Dope NFT Drop" \
  --project "dope-project" \
  --key "dope-drop-2024-q1" \
  --description "Dope owners can claim their rewards" \
  --network "ETH" \
  --contract "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589" \
  --entrypoint "0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589" \
  --args "TOKEN_ID,MERKLE_PROOF" \
  --data-file ./recipients.json
```

### Using Aliases

```bash
slot md c \
  --name "Community Rewards" \
  --project "community-dao" \
  --key "rewards-2024" \
  --network "STARKNET" \
  --contract "0x123..." \
  --entrypoint "0x456..." \
  --data-file ./community_rewards.json
```

### Minimal Example (No Optional Args)

```bash
slot merkle-drops create \
  --name "Simple Drop" \
  --project "test-project" \
  --key "simple-001" \
  --network "ETH" \
  --contract "0x123..." \
  --entrypoint "0x456..." \
  --data-file ./simple_drop.json
```

## Output

Upon successful creation, the command displays:

```
✅ Merkle Drop Created Successfully
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🏢 Details:
  • ID: merkle_drop_12345
  • Name: Dope NFT Drop
  • Project: dope-project
  • Key: dope-drop-2024-q1
  • Description: Dope owners can claim their rewards

🔗 Contract Details:
  • Network: ETH
  • Contract: 0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589
  • Entrypoint: 0x1dCD8763c01961C2BbB5ed58C6E51F55b1378589
  • Args: ["TOKEN_ID", "MERKLE_PROOF"]

🌳 Merkle Details:
  • Root: 0x8f7c9e2b1a5d4e8f3c6b9a2d7e1f4c8b5e9a3d7c1f8e4b2a6d9c3f7e1a5b8d2c6f
  • Entries: 3
  • Created: 2024-08-15T10:30:00Z
```

## Data Validation

The command performs comprehensive validation:

- ✅ **JSON Format**: Ensures data file is valid JSON
- ✅ **Array Structure**: Validates top-level array format
- ✅ **Entry Format**: Each entry must have exactly 2 elements
- ✅ **Address Format**: First element must be a string (address)
- ✅ **Token IDs**: Second element must be an array of integers

## Error Handling

Common error scenarios:

### Invalid Data Format
```bash
Entry 0 must have exactly 2 elements: [address, token_ids]
```

### Missing Required Parameters
```bash
error: the following required arguments were not provided:
  --project <PROJECT>
  --key <KEY>
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

## Best Practices

1. **Unique Keys**: Always use unique keys for each merkle drop to avoid conflicts
2. **Data Validation**: Validate recipient data before creation to avoid errors
3. **Backup Data**: Keep backups of your merkle drop data files
4. **Test First**: Test with small datasets before large-scale deployments

## Related Commands

- `slot auth login` - Authenticate with Slot API
- `slot teams` - Manage team/project settings

For more information, see the [Slot CLI documentation](https://docs.cartridge.gg/slot).