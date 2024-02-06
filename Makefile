DB=vector_database_backend

prepare:
	rustup target add wasm32-unknown-unknown

create:
	dfx create $(DB)

build:
	dfx build $(DB) --network=local

build-ic:
	dfx build $(DB) --network=ic

clippy:
	cargo clippy --all-targets -- -D warnings

clippy-fix:
	cargo clippy --fix --all-targets -- -D warnings

format-check: 
	cargo fmt -- --check

format: 
	cargo fmt

generate-did:
	scripts/generate-did.sh
	
generate: build
	make generate-did

start:
	dfx start --background --clean

create:
	dfx create canister $(DB)

deploy: build
	make generate-did
	dfx deploy $(DB)

redeploy:
	dfx deploy $(DB) --mode=reinstall
