IMAGE := casiros-api
COMPOSE := docker compose -f docker/docker-compose.yml

.PHONY: help run build docker-build docker-run docker-stop up up-d down logs

help:
	@echo "make run           - run the API server locally (cargo run)"
	@echo "make build         - build the workspace (cargo build)"
	@echo "make docker-build  - build the casiros-api Docker image"
	@echo "make docker-run    - run the Docker image standalone on :8080"
	@echo "make docker-stop   - stop/remove the standalone container"
	@echo "make up            - start the full stack (api + db + redis) via docker compose"
	@echo "make up-d          - same as 'up', but detached (runs in the background)"
	@echo "make down          - stop the docker compose stack"
	@echo "make logs          - follow logs from the docker compose stack"

# --- Backend (local, no Docker) ---------------------------------------------

run:
	cargo run -p casiros-api

build:
	cargo build --workspace

# --- Docker: standalone container -------------------------------------------

docker-build:
	docker build -f docker/Dockerfile -t $(IMAGE) .

docker-run: docker-build
	docker run --rm -p 8080:8080 --name $(IMAGE) $(IMAGE)

docker-stop:
	docker stop $(IMAGE)

# --- Docker Compose: full stack (api + db + redis) --------------------------

up:
	$(COMPOSE) up --build

up-d:
	$(COMPOSE) up --build -d

down:
	$(COMPOSE) down

logs:
	$(COMPOSE) logs -f
