#!/usr/bin/env bash

set -euo pipefail

# Runs API tests with a real Postgres database from infra/docker-compose.yml.
#
# Usage:
#   ./scripts/test-with-db.sh
#   ./scripts/test-with-db.sh --keep-db
#   ./scripts/test-with-db.sh --all
#
# Options:
#   --keep-db   Keep postgres running after tests
#   --all       Run full cargo test (default runs endpoint integration tests only)

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INFRA_DIR="$(cd "${ROOT_DIR}/../infra" && pwd)"
COMPOSE_FILE="${INFRA_DIR}/docker-compose.yml"
ENV_FILE="${ROOT_DIR}/.env"

KEEP_DB=0
RUN_ALL=0

for arg in "$@"; do
  case "$arg" in
    --keep-db) KEEP_DB=1 ;;
    --all) RUN_ALL=1 ;;
    *)
      echo "Unknown argument: $arg"
      exit 1
      ;;
  esac
done

if [[ ! -f "$COMPOSE_FILE" ]]; then
  echo "Compose file not found: $COMPOSE_FILE"
  exit 1
fi

if [[ ! -f "$ENV_FILE" ]]; then
  echo ".env not found at $ENV_FILE"
  exit 1
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "docker is required"
  exit 1
fi

if ! docker compose version >/dev/null 2>&1; then
  echo "docker compose is required"
  exit 1
fi

echo "Loading API env from $ENV_FILE"
set -a
source "$ENV_FILE"
set +a

export TEST_DATABASE_URL="${TEST_DATABASE_URL:-${DATABASE_URL:-}}"
if [[ -z "${TEST_DATABASE_URL}" ]]; then
  echo "TEST_DATABASE_URL (or DATABASE_URL) must be set"
  exit 1
fi

echo "Ensuring infra postgres is running..."
docker compose -f "$COMPOSE_FILE" up -d postgres

echo "Waiting for postgres health..."
for _ in {1..30}; do
  if docker compose -f "$COMPOSE_FILE" exec -T postgres pg_isready -U "${POSTGRES_USER:-pawtner}" -d "${POSTGRES_DB:-postgres}" >/dev/null 2>&1; then
    break
  fi
  sleep 2
done

if ! docker compose -f "$COMPOSE_FILE" exec -T postgres pg_isready -U "${POSTGRES_USER:-pawtner}" -d "${POSTGRES_DB:-postgres}" >/dev/null 2>&1; then
  echo "Postgres is not ready"
  exit 1
fi

cleanup() {
  if [[ "$KEEP_DB" -eq 0 ]]; then
    echo "Stopping infra postgres..."
    docker compose -f "$COMPOSE_FILE" stop postgres >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

cd "$ROOT_DIR"
if [[ "$RUN_ALL" -eq 1 ]]; then
  echo "Running full test suite..."
  cargo test
else
  echo "Running endpoint integration tests..."
  cargo test --test endpoints
fi

