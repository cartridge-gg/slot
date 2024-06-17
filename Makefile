BUILD_DIR := katana-genesis/build
CONTROLLER_CLASS_NAME := controller_CartridgeAccount
CONTROLLER_SUBMODULE := controller

${BUILD_DIR}: ${CONTROLLER_SUBMODULE}
	scarb --manifest-path $</packages/contracts/controller/Scarb.toml build
	mkdir -p $@
	mv $</target/**/${CONTROLLER_CLASS_NAME}* $@
