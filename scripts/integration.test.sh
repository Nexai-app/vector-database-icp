#!/usr/bin/bash

dfx identity use default
dfx identity remove owner
dfx identity import owner --seed-file keys/owner.seed
dfx identity use owner

dfx stop

dfx start --clean --background
dfx canister create vector_database_backend
dfx deploy vector_database_backend

npm run test