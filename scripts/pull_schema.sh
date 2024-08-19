#!/bin/bash

# https://github.com/graphql-rust/graphql-client/blob/main/graphql_client_cli/README.md
# cargo install graphql_client_cli

graphql-client introspect-schema --output slot/schema.json https://api.cartridge.gg/query
