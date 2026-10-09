#!/usr/bin/env python3
"""PostgreSQL regression tests for the migration and actual hybrid ranking SQL.

Run: python3 tests/waves_search.py
Uses a running pgvector container (WAVE_SEARCH_TEST_CONTAINER, default lenpostgres).
All fixtures, functions and migrations run in a transaction that is rolled back.
"""
from pathlib import Path
import os
import subprocess
import uuid

ROOT = Path(__file__).resolve().parents[1]
schema = "wave_test_" + uuid.uuid4().hex
fixture = (ROOT / "tests/waves_fixture.sql").read_text().replace("__TEST_SCHEMA__", schema)
migration = (ROOT / "migrations/V178__wave_hybrid_search.sql").read_text()
query = (ROOT / "src/waves/relevance.sql").read_text().replace(
    "/*ORDER*/", "match_priority DESC, score DESC, document_id ASC"
).replace("/*CANONICAL*/", (ROOT / "src/waves/canonical.sql").read_text())
function = """
CREATE FUNCTION test_search(text, text DEFAULT 'all', integer DEFAULT NULL,
                           vector DEFAULT NULL, double precision DEFAULT 0.4,
                           bigint DEFAULT 100, bigint DEFAULT 0)
RETURNS TABLE(total bigint, hits jsonb) LANGUAGE SQL AS $search$
""" + query + "$search$;\n"
assertions = (ROOT / "tests/waves_assertions.sql").read_text() + (ROOT / "tests/waves_ranking.sql").read_text()
script = fixture + migration + function + assertions + "\nROLLBACK;\n"
container = os.environ.get("WAVE_SEARCH_TEST_CONTAINER", "lenpostgres")
subprocess.run(["docker", "exec", "-i", container, "sh", "-c",
                'psql -q -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" -d "$POSTGRES_DB"'],
               input=script, text=True, check=True)
print("Discussion search PostgreSQL regression tests passed (rolled back).")
