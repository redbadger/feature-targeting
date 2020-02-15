default:
	cargo run

docker-build:
	docker build --tag redbadger/feature-targeting-adapter-istio .

docker-run:
	docker run -p 50051:50051 -it redbadger/feature-targeting-adapter-istio

.PHONY: default docker-build docker-run
