dev:
    topcoat dev --bin rusty-todos

migen name:
    cargo run --bin cli -- migration generate --name {{ name }}

migrate:
    cargo run --bin cli -- migration apply

drop:
    cargo run --bin cli -- migration drop

reset:
    cargo run --bin cli -- migration reset
