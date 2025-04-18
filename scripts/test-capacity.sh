#!/bin/sh

echo "creating..."

total=100
batch_size=10
max_retries=3

# Function to retry commands
retry_command() {
    local cmd="$1"
    local retries=0

    while [ $retries -lt $max_retries ]; do
        if eval "$cmd"; then
            return 0  # Command succeeded
        else
            retries=$((retries + 1))
            if [ $retries -eq $max_retries ]; then
                echo "Failed after $max_retries attempts: $cmd"
                return 1
            fi
            echo "Attempt $retries failed, retrying..."
            sleep 2  # Wait before retrying
        fi
    done
}

# Function to process create commands
process_create_batch() {
    local start=$1
    local end=$2

    for i in $(seq $start $end); do
        cmd="slot d create --tier epic \"ls-tmpx-$i\" katana"
        retry_command "$cmd" &
        sleep 1
    done
    wait
}

# Function to process delete commands
process_delete_batch() {
    local start=$1
    local end=$2

    for i in $(seq $start $end); do
        cmd="slot d delete \"ls-tmpx-$i\" katana -f"
        retry_command "$cmd" &
        sleep 1
    done
    wait
}

# Create deployments in batches
for ((i=1; i<=total; i+=batch_size)); do
    end=$((i+batch_size-1))
    if [ $end -gt $total ]; then
        end=$total
    fi
    echo "Processing create batch $i to $end..."
    process_create_batch $i $end
done

echo "success"

echo "tearing down..."

# Delete deployments in batches
for ((i=1; i<=total; i+=batch_size)); do
    end=$((i+batch_size-1))
    if [ $end -gt $total ]; then
        end=$total
    fi
    echo "Processing delete batch $i to $end..."
    process_delete_batch $i $end
done

echo "success"
