.PHONY: default
default: install ## Install todomvc-web

.PHONY: install
install: ## Install todomvc-web
	jq -r ".web|to_entries|map(\"\(.key)=\(.value|tostring)\")|.[]" ../client-secret.json | \
		kubectl create secret generic oauth-creds --dry-run=client -o yaml --from-env-file /dev/stdin >secret.yaml
	kustomize build . | kubectl apply -f -

.PHONY: delete
delete: ## Remove todomvc-web
	kustomize build . | kubectl delete -f - --ignore-not-found || true

.PHONY: help
help: ## Display this help screen
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
