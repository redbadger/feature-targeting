.PHONY: default
default:
	(cd client && wasm-pack build --target web)
	docker build -t todomvc_web_server .
