#!/usr/bin/bash

dfx identity use owner
dfx canister create vector_database_backend
dfx deploy vector_database_backend