#!/bin/bash

for n in {1..100}; do
    cargo run -- deployments delete "ls-tmp-$n" torii -f
    cargo run -- deployments delete "ls-tmp-$n" katana -f
    cargo run -- deployments delete "ls-tmp-$n" saya -f
done
