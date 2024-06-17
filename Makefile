BUILD_DIR := katana-genesis/build

generate-artifacts: controller
	scarb --manifest-path $</crates/cartridge_account/Scarb.toml build
	mkdir -p ${BUILD_DIR}
	# Maybe only move the exact class that we want
	mv $</target/dev/* ${BUILD_DIR}

.PHONY: generate-artifacts
