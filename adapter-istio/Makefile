default: build
	cargo run

build: template/config.proto_descriptor template/generated_template.proto_descriptor
	cargo build

release: template/config.proto_descriptor template/generated_template.proto_descriptor
	cargo build --release

docker-build:
	docker build --tag redbadger/feature-targeting-adapter-istio .

docker-run:
	docker run -p 50051:50051 -it redbadger/feature-targeting-adapter-istio


# Adapter interface protobuf compilation (here be dragons)

protobuf-install: template/feature-targeting-template.yaml template/feature-targeting.yaml
	cp template/*.yaml ../examples/2-enable-adapter/

protobuf-clean:
	rm -f template/*.yaml template/*.proto_descriptor template/generated_template.*

## K8s manifest files

template/feature-targeting.yaml: template/config.proto_descriptor
	cd template && mixgen adapter \
		--config config.proto_descriptor \
		--session_based=false \
		--name feature-targeting \
		--description "Mixer adapter for feature targeting" \
		--templates feature-targeting \
		--output ./

template/feature-targeting-template.yaml: template/generated_template.proto_descriptor
	cd template && mixgen template \
		--descriptor generated_template.proto_descriptor \
		--name feature-targeting \
		--output feature-targeting-template.yaml

## Mixgen generated protobuf

template/generated_template.proto_descriptor: template/generated_template.proto
	protoc -o template/generated_template.proto_descriptor \
		-I ./template \
		-I ./template/proto \
		-I ./template/proto/common-protos \
		--include_imports \
		template/generated_template.proto

template/generated_template.proto: template/template.proto_descriptor
	cd template && mixgen api -t template.proto_descriptor --go_out /dev/null

## Descriptors needed for mixgen

template/template.proto_descriptor: template/template.proto template/proto/proto.lock
	protoc -o template/template.proto_descriptor \
		-I ./template \
		-I ./template/proto \
		-I ./template/proto/common-protos \
		--include_imports \
		template/template.proto

template/config.proto_descriptor: template/config.proto
	protoc -o template/config.proto_descriptor \
		--include_imports \
		template/config.proto

## Protobuf dependencies

template/proto/proto.lock:
	cd template && ./get-proto.sh

.PHONY: default build release docker-build docker-run protobuf-install protobuf-clean
