app := feature-targeting-operator
image := red-badger.com/$(app)
tag := latest

.PHONY: default
default: ## Run locally
	cargo run

.PHONY: test
test: ## Run tests [TEST=test_name (optional)]
	cargo test $$TEST && cargo clippy

.PHONY: ci-branch
ci-branch: docker-build ## Build branch in CI

.PHONY: ci-master
ci-master: ci-branch docker-push ## Build master in CI

.PHONY: docker-build
docker-build: ## Build and tag Docker image (with search-client create as dependency)
	DOCKER_BUILDKIT=1 docker build . \
		--progress plain \
		--tag $(app) \
		--tag $(image) \
		--tag $(image):$(tag)

.PHONY: docker-run
docker-run: ## Run Docker image
	docker run -it $(image)

.PHONY: docker-pull
docker-pull: ## Pull Docker image
	docker pull $(image):$(tag)

.PHONY: docker-push
docker-push: ## Push Docker image
	docker push $(image)
	docker push $(image):$(tag)

.PHONY: help
help: ## Display this help screen
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
