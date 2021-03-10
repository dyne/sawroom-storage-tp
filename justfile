default:
  @just --list --list-prefix " 🦾 "

start:
  docker-compose down
  docker-compose up --build

bash:
  docker exec -it sawtooth-shell-default bash

lint:
  cargo clippy

lint-fix:
  cargo clippy --fix -Z unstable-options
