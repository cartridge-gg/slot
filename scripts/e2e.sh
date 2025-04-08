#!/bin/sh

set -eux

rand=$(date +%s)

project="slot-e2e-infra-$rand"

cargo run -- d create "$project" katana

sleep 10

res=$(curl --request POST -s \
     --url "https://api.cartridge.gg/x/$project/katana" \
     --header 'accept: application/json' \
     --header 'content-type: application/json' \
     --data '
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "starknet_specVersion"
}
')

if ! echo "$res" | grep -q '"jsonrpc":"2.0"'; then
  echo "katana failed, response: $res"
  cargo run -- d delete "$project" katana -f || true
  exit 1
fi

cargo run -- d delete "$project" katana -f

# test torii

echo 'world_address = "0x585a28495ca41bece7640b0ccf2eff199ebe70cc381fa73cb34cc5721614fbd"\nrpc = "https://api.cartridge.gg/x/starknet/sepolia"' > /tmp/config.toml

cargo run -- d create "$project" torii --config /tmp/config.toml

sleep 10

res=$(curl --request POST -s \
  --url "https://api.cartridge.gg/x/$project/torii/graphql" \
  --data '
{
  "query": "query { models { edges { node { id } } } }"
}
')

if ! echo "$res" | grep -q '"models":'; then
  echo "torii failed, response: $res"
  cargo run -- d delete "$project" torii -f || true
  exit 1
fi

cargo run -- d delete "$project" torii -f

echo "e2e test passed"
