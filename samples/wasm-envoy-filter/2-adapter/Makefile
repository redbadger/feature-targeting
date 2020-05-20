.PHONY: default
default: install ## Install feature_targeting filter

.PHONY: install
install: ## Install feature_targeting filter
	kustomize build . | kubectl apply -f -

.PHONY: delete
delete: ## Remove feature_targeting filter
	kustomize build . | kubectl apply --ignore-not-found -f - || true

.PHONY: help
help: ## Display this help screen
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
