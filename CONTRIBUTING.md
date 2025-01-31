# Contributing to Slot

## Architecture

The commands that Slot CLI offers must reflect what's the internal infrastructure is expecting.
The current design is the following:

1. Slot CLI implements infra specific commands (like accounts, teams, etc..). Those one are directly mapped to the infra specific code.
2. To use the services like Katana, Torii, etc.. Slot CLI must know the arguments that each service expects. For this, Slot CLI is using the `cli` crate for each service.
3. When creating or updating a service like Katana and Torii, Slot CLI will gather the arguments from the CLI and creates a TOML configuration file for each service, which will ease the process of creating the services and passing arguments to the corresponding service.

## GraphQL

Slot CLI is using a GraphQL API to interact with the Slot API (infrastructure).

When you have to modify a GraphQL query, you must update the corresponding `.graphql` file and regenerate the Rust code by running:
```bash
# Install the graphql-client CLI.
# https://github.com/graphql-rust/graphql-client/blob/main/graphql_client_cli/README.md
cargo install graphql-client

# Pull the latest schema from the Slot API (or use the pull_schema.sh script)
graphql-client introspect-schema --output slot/schema.json https://api.cartridge.gg/query

# Then regenerate the Rust code for the queries that you have modified.
# Using this command actually change the macro to something auto-generated. And we prefer macro expansion.
# graphql-client generate --schema-path slot/schema.json slot/src/graphql/deployments/update.graphql
```

## Local Development

### Pointing to a local API

Run the cartridge api locally.

Then set these variables in your slot directory:

```shell
export CARTRIDGE_API_URL=http://localhost:8000
export CARTRIDGE_KEYCHAIN_URL=http://localhost:3001
```

Then run `cargo run -- <cmd>`.
