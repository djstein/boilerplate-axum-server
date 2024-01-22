# Makefile with phony targets

.PHONY: setup lint format setup-hooks services
OS := $(shell uname)

runserver: ## Run Development Server
	@cargo-watch -q -w . -x "run"
