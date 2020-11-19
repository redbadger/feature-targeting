localdb := postgres://postgres:postgres@localhost:5432

.PHONY: default
default:
	cargo run -- --database-url=$(localdb)


.PHONY: docker
docker: prepare
	docker build -t todomvc_api .

.PHONY: prepare
prepare: migrate-local ## Prepares new version of sqlx-data.json
	cargo clean --package todomvc_api && DATABASE_URL=$(localdb) cargo sqlx prepare -- --bin todomvc_api

.PHONY: reset-local
reset-local: ## Resets local database
	docker-compose down --volumes

.PHONY: migrate-local
migrate-local: ## Bring up local dependencies, run DB migrations
	docker-compose up -d
	until DATABASE_URL=$(localdb) cargo sqlx migrate run; do sleep 1; done
