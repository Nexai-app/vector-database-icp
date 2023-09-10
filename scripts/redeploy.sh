#!/usr/bin/bash

dfx identity import owner --seed-file keys/owner.seed
dfx identity use owner
dfx canister create vector_database_backend
dfx deploy vector_database_backend