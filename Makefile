build:
	dfx build vector_database_backend

generate:
	scripts/generate-did.sh

redeploy:
	dfx deploy vector_database_backend --mode=reinstall
	