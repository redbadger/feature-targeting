.PHONY: default
default:
	cargo run -- --database-url=postgres://stuartharris@localhost/todos


.PHONY: docker
docker: prepare
	docker build -t todomvc_api .

.PHONY: prepare
prepare:
	rm -f sqlx-data.json
	cargo sqlx prepare -- --lib
