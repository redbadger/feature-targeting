.PHONY: default
default: client server

.PHONY: docker
docker: client
	docker build -t todomvc_web_server .

.PHONY: client
client:
	cd client && make

.PHONY: server
server:
	cd server && make
