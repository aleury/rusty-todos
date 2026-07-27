dev:
    topcoat dev --bin rusty-todos

generate-migration name:
    cargo run --bin cli -- migration generate --name {{ name }}

migrate:
    cargo run --bin cli -- migration apply

drop:
    cargo run --bin cli -- migration apply

reset:
    cargo run --bin cli -- migration reset
