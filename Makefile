.PHONY: e2e-test smoke-test

e2e-test:
	@echo "Running end-to-end event lifecycle test..."
	@bash scripts/e2e_event_lifecycle.sh

# Closes #558: deploy to testnet and verify basic operations.
smoke-test:
	@echo "Running testnet smoke test..."
	@bash scripts/smoke_test.sh