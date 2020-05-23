.PHONY: default
default: install ## Install echo-service

.PHONY: install
install: ## Install echo-service
	kustomize build . | kubectl apply -f -

.PHONY: delete
delete: ## Remove echo-service
	kustomize build . | kubectl delete -f - --ignore-not-found || true

.PHONY: help
help: ## Display this help screen
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
