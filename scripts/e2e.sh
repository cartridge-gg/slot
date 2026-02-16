#!/bin/sh

set -eux

# Creates a simple Torii config file.
create_torii_config() {
    local config_path=$1
    cat > "$config_path" << EOF
world_address = "0x585a28495ca41bece7640b0ccf2eff199ebe70cc381fa73cb34cc5721614fbd"
rpc = "https://api.cartridge.gg/x/starknet/sepolia"
EOF
}

# Creates a simple Katana config file.
create_katana_config() {
    local config_path=$1
    cat > "$config_path" << EOF
block_time = 5000
EOF
}

# Checks if Katana is ready and responding.
check_katana() {
    local project=$1
    local max_retries=3
    local retry_count=0
    local res

    while [ $retry_count -lt $max_retries ]; do
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

        # Check for both valid JSON-RPC response and absence of timeout
        if echo "$res" | grep -q '"jsonrpc":"2.0"' && ! echo "$res" | grep -q '"error"'; then
            return 0
        fi

        retry_count=$((retry_count + 1))
        if [ $retry_count -lt $max_retries ]; then
            echo "Katana check attempt $retry_count failed, retrying in 5 seconds..."
            sleep 5
        fi
    done

    echo "Katana check failed after $max_retries attempts, last response: $res"
    return 1
}

# Checks if Torii is ready and responding.
check_torii() {
    local project=$1
    local max_retries=3
    local retry_count=0
    local res

    while [ $retry_count -lt $max_retries ]; do
        res=$(curl --request POST -s \
          --url "https://api.cartridge.gg/x/$project/torii/graphql" \
          --data '
{
  "query": "query { models { edges { node { id } } } }"
}
')

        if echo "$res" | grep -q '"models":'; then
            return 0
        fi

        retry_count=$((retry_count + 1))
        if [ $retry_count -lt $max_retries ]; then
            echo "Torii check attempt $retry_count failed, retrying in 5 seconds..."
            sleep 5
        fi
    done

    echo "Torii check failed after $max_retries attempts, last response: $res"
    return 1
}

# Tests Katana deployment creation, tier update, config update and deletion.
test_katana() {
    local project=$1
    local config_path=$2

    create_katana_config "$config_path"
    cargo run -- d create "$project" katana --config "$config_path"

    sleep 10

    if ! check_katana "$project"; then
        cargo run -- d delete "$project" katana -f || true
        return 1
    fi

    cargo run -- d update --tier basic "$project" katana
    sleep 20

    if ! check_katana "$project"; then
        cargo run -- d delete "$project" katana -f || true
        return 1
    fi

    create_katana_config "$config_path"
    cargo run -- d update "$project" katana --config "$config_path"
    # Faster update, since it's only a restart with the new config
    # And the slot request is waiting the pod to be ready to return.
    sleep 20

    if ! check_katana "$project"; then
        cargo run -- d delete "$project" katana -f || true
        return 1
    fi

    cargo run -- d delete "$project" katana -f
    return 0
}

# Tests Torii deployment creation, tier update, config update and deletion.
test_torii() {
    local project=$1
    local config_path=$2

    create_torii_config "$config_path"
    cargo run -- d create "$project" torii --config "$config_path"

    sleep 15

    if ! check_torii "$project"; then
        cargo run -- d delete "$project" torii -f || true
        return 1
    fi

    cargo run -- d update --tier basic "$project" torii
    sleep 20

    if ! check_torii "$project"; then
        cargo run -- d delete "$project" torii -f || true
        return 1
    fi

    create_torii_config "$config_path"
    cargo run -- d update "$project" torii --config "$config_path"
    # Faster update, since it's only a restart with the new config.
    # And the slot request is waiting the pod to be ready to return.
    sleep 20

    if ! check_torii "$project"; then
        cargo run -- d delete "$project" torii -f || true
        return 1
    fi

    cargo run -- d delete "$project" torii -f
    return 0
}

# Main execution
main() {
    local rand
    rand=$(date +%s)
    local project="slot-e2e-infra-$rand"
    local torii_config="/tmp/torii.toml"
    local katana_config="/tmp/katana.toml"

    if ! test_katana "$project" "$katana_config"; then
        echo "Katana test failed"
        exit 1
    fi

    if ! test_torii "$project" "$torii_config"; then
        echo "Torii test failed"
        exit 1
    fi

    echo "e2e tests passed"
}

main
