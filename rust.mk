
.PHONY: fmt
fmt:
	cargo fmt
	git add -u
	cargo clippy --fix --allow-staged

.PHONY: check-fmt
check-fmt:
	cargo fmt --check
	cargo clippy
