#!/bin/bash

# https://github.com/graphql-rust/graphql-client/blob/main/graphql_client_cli/README.md
# cargo install graphql_client_cli

# check if param to pull schema is set to "local"
if [ "$1" = "local" ]; then
	graphql-client introspect-schema --output slot/schema.json http://localhost:8000/query
elif [ "$1" = "prod" ]; then
	graphql-client introspect-schema --output slot/schema.json https://api.cartridge.gg/query
else
	echo "usage: pull_schema.sh [local|prod]"
fi
