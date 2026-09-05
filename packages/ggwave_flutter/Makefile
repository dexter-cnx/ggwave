ROOT := ../..

.PHONY: help doctor deps frb-install frb-generate bootstrap prepare-android clean clean-generated analyze test preflight run-android build-android apk-check

help doctor deps frb-install frb-generate bootstrap prepare-android clean clean-generated analyze test preflight run-android build-android apk-check:
	$(MAKE) -C $(ROOT) $@ DEVICE="$(DEVICE)" TARGET="$(TARGET)"
