all:
	sam build -u -bi build-rust-provided.al2

MODE ?= debug
build-TodaysMenuApiFunction: build-TodaysMenuApiFunction-$(MODE)

build-TodaysMenuApiFunction-release:
	cargo build --release
	cp ./target/release/todays-menu-api $(ARTIFACTS_DIR)/bootstrap

build-TodaysMenuApiFunction-debug:
	cargo build
	cp ./target/debug/todays-menu-api $(ARTIFACTS_DIR)/bootstrap

docker-image:
	docker build -t build-rust-provided.al2 docker

apigw:
	sam local start-api -p 8080 --docker-network todays-menu-api_default --env-vars env.json

localstack:
	docker-compose -f docker/docker-compose.yml --project-name todays-menu-api up --build -d

stop-localstack:
	docker-compose -f docker/docker-compose.yml --project-name todays-menu-api down