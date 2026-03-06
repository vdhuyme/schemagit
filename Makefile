.PHONY: fmt-toml
fmt-toml:
	@cargo run --package cargo-fmt-toml

.PHONY: check-fmt-toml
check-fmt-toml:
	@cargo run --package cargo-fmt-toml -- --check
