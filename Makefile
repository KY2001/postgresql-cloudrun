GIT_SHA := $(shell git rev-parse HEAD 2>/dev/null)

# Cloud Run
GCP_REGION ?= asia-northeast1
AR_REPO := $(GCP_REGION)-docker.pkg.dev/$(SERVER_PROJECT_ID)/$(SERVER_NAME)
IMAGE := $(AR_REPO)/$(SERVER_NAME)

# Task
.PHONY: gen
gen:
	rm -rf openapi/src
	docker run --rm --user $(shell id -u):$(shell id -g) --volume $(CURDIR):/local \
		openapitools/openapi-generator-cli:v7.16.0 generate -i /local/openapi/openapi.yaml -g rust-axum \
		-o /local/openapi --additional-properties=packageName=openapi,hideGenerationTimestamp=true

.PHONY: build
build:
	cargo build --release --locked

.PHONY: format
format:
	cargo fmt
	terraform fmt -recursive terraform/

.PHONY: lint
lint:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	tflint --chdir=terraform/ --recursive
	docker run --rm -i hadolint/hadolint < Dockerfile

.PHONY: lint-fix
lint-fix: format
	cargo clippy --fix --allow-dirty --allow-staged

.PHONY: all
all: gen lint-fix lint build

.PHONY: upgrade-packages
upgrade-packages:
	cargo update

.PHONY: docker-build-and-run
docker-build-and-run: docker-build docker-run

.PHONY: docker-build
docker-build:
	docker build . --platform linux/amd64 --tag $(SERVER_NAME) --load --progress=plain

.PHONY: docker-run
docker-run:
	mkdir -p data/backup && chmod 777 data/backup
	docker container run --rm --interactive --tty --tmpfs /data:mode=1777 \
		--env PGBACKREST_REPO1_TYPE=posix --env PGBACKREST_REPO1_PATH=/backup --volume $(CURDIR)/data/backup:/backup \
		-p $(SERVER_PORT):8080 $(SERVER_NAME)

.PHONY: gcloud-ar-login
gcloud-ar-login:
	gcloud auth configure-docker $(GCP_REGION)-docker.pkg.dev --quiet

.PHONY: gcloud-build-and-push
gcloud-build-and-push:
	docker build . --platform linux/amd64 --tag $(IMAGE):$(SERVER_ENV)-$(GIT_SHA) --progress plain
	docker tag $(IMAGE):$(SERVER_ENV)-$(GIT_SHA) $(IMAGE):$(SERVER_ENV)-latest
	docker push $(IMAGE):$(SERVER_ENV)-$(GIT_SHA)
	docker push $(IMAGE):$(SERVER_ENV)-latest

.PHONY: gcloud-deploy
gcloud-deploy:
	gcloud beta run services update $(SERVER_NAME) --project=$(SERVER_PROJECT_ID) --region=$(GCP_REGION) --image=$(IMAGE):$(SERVER_ENV)-$(GIT_SHA)
