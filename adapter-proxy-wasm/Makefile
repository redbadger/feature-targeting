

build-image: release ./feature_targeting.wasm
	wasme build precompiled feature_targeting.wasm --tag feature-targeting:0.1.0

install: release ./feature_targeting.wasm
	mkdir -p /tmp/envoy-filters
	cp ./feature_targeting.wasm /tmp/envoy-filters/feature_targeting.wasm

# Running with istio/proxyv2:1.5.x is key
run-local:
	wasme deploy envoy feature-targeting:0.1.0 \
		--envoy-image=istio/proxyv2:1.5.1 \
		--bootstrap=envoy-bootstrap.yml \
		--config='$(shell cat filter-config.json)'

release:
	cargo build --target wasm32-unknown-unknown --release

./feature_targeting.wasm: target/wasm32-unknown-unknown/release/adapter_proxy_wasm.wasm
	cp target/wasm32-unknown-unknown/release/adapter_proxy_wasm.wasm ./feature_targeting.wasm

./target/wasm32-unknown-unknown/release/adapter_proxy_wasm.wasm: release

.PHONY: release run-local show-config
