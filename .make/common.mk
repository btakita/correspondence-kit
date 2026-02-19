init: PY_VERSION = $(shell [ -f .python-version ] && \
	cat .python-version || \
	uv run python -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')" \
)
init:
	@echo "Using Python version: $(PY_VERSION)"

	@if command -v mise >/dev/null 2>&1; then \
		mise install; \
	fi

	uv venv .venv --python "$(PY_VERSION)" --no-project --clear --seed $(VENV_ARGS)

	@if [ -n "$(ALL)" ]; then \
		uv sync --python "$(PY_VERSION)" --all-groups --all-extras $(SYNC_ARGS); \
	else \
		uv sync --python "$(PY_VERSION)" $(SYNC_ARGS); \
	fi
