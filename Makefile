BUILD_DIR := katana-genesis/build
CONTROLLER_CLASS_NAME := controller_CartridgeAccount
CONTROLLER_SUBMODULE := controller

all: ${BUILD_DIR}

${BUILD_DIR}: ${CONTROLLER_SUBMODULE}
	scarb --manifest-path $</packages/contracts/controller/Scarb.toml build
	mkdir -p $@
	mv $</target/dev/${CONTROLLER_CLASS_NAME}* $@

test: ${BUILD_DIR}
	cargo test --all-features --workspace

.phony: test
