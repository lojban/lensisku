#!/usr/bin/env bash
# Migration + backend regressions against an isolated PostgreSQL fixture.
set -euo pipefail
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
rafsi_test_container="lensisku-rafsi-test-$$"
trap 'docker rm -f "$rafsi_test_container" >/dev/null 2>&1 || true' EXIT
docker run --rm -d --name "$rafsi_test_container" \
  -e POSTGRES_HOST_AUTH_METHOD=trust -v "$repo_root:/workspace:ro" \
  -p 127.0.0.1::5432 postgres:17 >/dev/null
for attempt in {1..30}; do
  if docker exec "$rafsi_test_container" pg_isready -U postgres >/dev/null 2>&1; then break; fi
  sleep 1
done
rafsi_test_port="$(docker port "$rafsi_test_container" 5432/tcp | cut -d: -f2)"
cd "$repo_root"
RAFSI_TEST_DATABASE_URL="postgres://postgres@127.0.0.1:$rafsi_test_port/postgres" \
  cargo test --offline --bin lensisku rafsi_ -- --include-ignored
