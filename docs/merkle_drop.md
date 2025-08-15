# Merkle Drop Commands

The Slot CLI provides commands to create and manage merkle drops for token distribution campaigns.

## Overview

Merkle drops are an efficient way to distribute tokens to a large number of recipients while minimizing gas costs. The system uses a merkle tree to prove eligibility for claiming tokens without storing all recipient data on-chain.

The merkle root is automatically calculated server-side from the provided claims data, ensuring consistency and eliminating the need for manual merkle tree generation.

## Commands

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

- `--team <TEAM>` - Team name to associate the merkle drop with
- `--key <KEY>` - Unique key for the merkle drop
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
slot merkle-drops create preset --name <PRESET> --key <KEY> --team <TEAM> [--network <NETWORK>]
```

##### Required Parameters

- `--name <PRESET>` - Preset name from [cartridge-gg/presets](https://github.com/cartridge-gg/presets/tree/main/configs)
- `--key <KEY>` - Merkle drop key from the preset
- `--team <TEAM>` - Team name to associate the merkle drop with

##### Optional Parameters

- `--network <NETWORK>` - Network to use from preset (default: SN_MAIN)

## Data File Formats

### Parameters Method - Data File Format

For the `params` method, the data file must be a JSON array where each entry contains:
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

### JSON Method - Configuration File Format

For the `json` method, the configuration file must contain both the merkle drop configuration and the recipient data:

```json
{
  "key": "my-drop-2024",
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
      [1, 5352, 5533, 7443]
    ],
    [
      "0x1234567890123456789012345678901234567890",
      [100, 200, 300]
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
  --team "dope-team" \
  --key "dope-drop-2024-q1" \
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
  --team "community-dao" \
  --key "rewards-2024" \
  --network "STARKNET" \
  --contract "0x123..." \
  --entrypoint "0x456..." \
  --data-file ./community_rewards.json
```

#### Minimal Example (No Optional Args)

```bash
slot merkle-drops create params \
  --team "test-team" \
  --key "simple-001" \
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
    ["0x1234...", [100, 200]],
    ["0x5678...", [50, 75]]
  ]
}
```

### Method 3: From Community Presets

#### Using Dope Wars Preset

```bash
slot merkle-drops create preset \
  --name "dope-wars" \
  --key "dope" \
  --team "my-team" \
  --network "SN_MAIN"
```

#### Using Custom Preset

```bash
slot merkle-drops create preset \
  --name "my-community-preset" \
  --key "season-1-rewards" \
  --team "community-dao"
```

#### Preset with Different Network

```bash
slot merkle-drops create preset \
  --name "dope-wars" \
  --key "dope" \
  --team "my-team" \
  --network "ETH"
```

## Output

Upon successful creation, the command displays:

```
✅ Merkle Drop Created Successfully
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🏢 Details:
  • ID: merkle_drop_12345
  • Team: dope-team
  • Key: dope-drop-2024-q1
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
  --team <TEAM>
  --key <KEY>
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
4. Use the merkle drop key from the configuration

### Preset Structure Example

Preset `dope-wars` contains:
```
configs/dope-wars/
├── config.json                 # Main preset configuration
└── merkledrops/
    └── dope.json              # Merkle drop data for key "dope"
```

## Best Practices

1. **Unique Keys**: Always use unique keys for each merkle drop to avoid conflicts
2. **Method Selection**: 
   - Use `params` for one-off drops with custom configuration
   - Use `json` for complex drops with version control needs
   - Use `preset` for community-standard drops
3. **Data Validation**: Validate recipient data before creation to avoid errors
4. **Backup Data**: Keep backups of your merkle drop data files
5. **Test First**: Test with small datasets before large-scale deployments
6. **Preset Updates**: When using presets, check for updates in the community repository

## Related Commands

- `slot auth login` - Authenticate with Slot API
- `slot teams` - Manage team/project settings

For more information, see the [Slot CLI documentation](https://docs.cartridge.gg/slot).
