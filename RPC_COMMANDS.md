# RPC Commands Implementation

This document outlines the CLI commands added for RPC API key management and CORS domain whitelisting.

## Commands Structure

### Main Command: `slot rpc`
```bash
slot rpc --help
```

### RPC API Key Management: `slot rpc tokens`
```bash
# Create a new RPC API key
slot rpc tokens create <KEY_NAME> --team <TEAM_NAME>

# Delete an RPC API key
slot rpc tokens delete <KEY_ID> --team <TEAM_NAME>

# List all RPC API keys for a team (temporarily disabled)
slot rpc tokens list --team <TEAM_NAME>
```

### CORS Domain Whitelist Management: `slot rpc whitelist`
```bash
# Add a domain to the CORS whitelist
slot rpc whitelist add <DOMAIN> --team <TEAM_NAME>

# Remove a domain from the CORS whitelist
slot rpc whitelist remove <ENTRY_ID> --team <TEAM_NAME>

# List all whitelisted domains for a team (temporarily disabled)
slot rpc whitelist list --team <TEAM_NAME>
```

## Implementation Status

✅ **Complete**: CLI command structure and argument parsing
✅ **Complete**: Command registration and help system  
✅ **Complete**: GraphQL integration for create/delete operations
⚠️ **Limited**: List operations temporarily disabled due to complex GraphQL connection types

## Files Created/Modified

### CLI Command Structure
- `cli/src/command/rpc/mod.rs` - Main RPC command enum
- `cli/src/command/rpc/tokens.rs` - Token management subcommands
- `cli/src/command/rpc/whitelist.rs` - Whitelist management subcommands
- `cli/src/command/rpc/tokens/create.rs` - Create token command
- `cli/src/command/rpc/tokens/delete.rs` - Delete token command
- `cli/src/command/rpc/tokens/list.rs` - List tokens command
- `cli/src/command/rpc/whitelist/add.rs` - Add whitelist origin command
- `cli/src/command/rpc/whitelist/remove.rs` - Remove whitelist origin command
- `cli/src/command/rpc/whitelist/list.rs` - List whitelist origins command

### GraphQL Schema Files
- `slot/src/graphql/rpc/create_token.graphql` - Create token mutation
- `slot/src/graphql/rpc/delete_token.graphql` - Delete token mutation
- `slot/src/graphql/rpc/list_tokens.graphql` - List tokens query
- `slot/src/graphql/rpc/add_whitelist_origin.graphql` - Add whitelist origin mutation
- `slot/src/graphql/rpc/remove_whitelist_origin.graphql` - Remove whitelist origin mutation
- `slot/src/graphql/rpc/list_whitelist_origins.graphql` - List whitelist origins query
- `slot/src/graphql/rpc/mod.rs` - GraphQL query/mutation definitions

### Core Registration
- `cli/src/command/mod.rs` - Added RPC command to main CLI
- `slot/src/graphql/mod.rs` - Added RPC GraphQL module (commented out)

## Working Commands Demonstration

The implementation has been successfully tested with a live API. Here's a complete workflow:

### 1. Create a Team
```bash
$ slot teams test-team create --email test@example.com
Team test-team created successfully 🚀
```

### 2. Create RPC API Keys
```bash
$ slot rpc tokens create my-app-key --team test-team
✅ RPC API Key Created Successfully
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔑 Details:
  • ID: cmffs3a9p0004mmozw61h39tt
  • Name: my-app-key
  • Team: test-team
  • Created: 2025-09-11T20:05:20.317316+01:00

🔐 Secret Key:
  • sk_5e13252f527c3146b9bd3021b8837524

⚠️  Important: Save this secret key securely - it won't be shown again!
🔍 Key Prefix (for identification): sk_5e132
```

### 3. Add CORS Domains
```bash
$ slot rpc whitelist add https://myapp.com --team test-team
✅ Origin Added to CORS Whitelist Successfully
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🌐 Details:
  • ID: cmffs3fbu0005mmoz1ajehcoq
  • Domain: https://myapp.com
  • Team: test-team
  • Created: 2025-09-11T20:05:26.874251+01:00

$ slot rpc whitelist add "*.mycompany.com" --team test-team
✅ Origin Added to CORS Whitelist Successfully
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🌐 Details:
  • ID: cmffs4j0a0008mmozn01vm2bt
  • Domain: *.mycompany.com
  • Team: test-team
  • Created: 2025-09-11T20:06:18.298939+01:00
```

### 4. Delete Resources
```bash
$ slot rpc tokens delete cmffs3a9p0004mmozw61h39tt --team test-team
✅ RPC API Key Deleted Successfully
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🗑️  API Key ID cmffs3a9p0004mmozw61h39tt has been removed from team 'test-team'

$ slot rpc whitelist remove cmffs3fbu0005mmoz1ajehcoq --team test-team
✅ Origin Removed from CORS Whitelist Successfully
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🗑️  CORS domain ID cmffs3fbu0005mmoz1ajehcoq has been removed from team 'test-team'
```

## Future Enhancements

### Re-enable List Commands
To restore list functionality, the GraphQL connection types need to be properly handled:

1. **Add proper type imports** for `BigInt`, `Long`, and complex connection types
2. **Implement pagination** for large result sets  
3. **Add filtering options** by team, active status, etc.

The schema supports these operations via:
- `rpcApiKeys(first: Int, after: String, where: RPCApiKeyWhereInput)`
- `rpcCorsDomains(first: Int, after: String, where: RPCCorsDomainWhereInput)`

## Technical Implementation Details

### GraphQL Schema Integration
The commands use the actual GraphQL schema with these mutations:
- `createRpcApiKey(teamName: String!, name: String!)` - Creates API keys
- `deleteRpcApiKey(id: ID!)` - Deletes API keys  
- `createRpcCorsDomain(teamName: String!, domain: String!, rateLimitPerMinute: Int)` - Adds CORS domains
- `deleteRpcCorsDomain(id: ID!)` - Removes CORS domains

### Key Features
- **Secure Key Generation**: API keys are generated server-side with proper entropy
- **Key Prefix Display**: Shows first 5 characters for easy identification
- **Rate Limiting**: CORS domains support configurable rate limits (default: 60/min)
- **Wildcard Support**: CORS domains accept wildcard patterns like `*.example.com`
- **Team Isolation**: All operations are scoped to specific teams

### Error Handling
Commands provide clear error messages for common scenarios:
- Invalid team names
- Non-existent resource IDs
- Permission errors
- Network connectivity issues

The CLI follows the established patterns in the codebase and integrates seamlessly with the existing command structure.