import assert from "node:assert/strict";
import { promises as fs } from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { publishFrontend } from "./publish-frontend.mjs";

const DAY = 24 * 60 * 60 * 1000;

async function fixture(t) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "lensisku-publish-"));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  const staging = path.join(root, "staging");
  const dist = path.join(root, "dist");
  await fs.mkdir(path.join(staging, "assets"), { recursive: true });
  await fs.mkdir(path.join(dist, "assets"), { recursive: true });
  await fs.writeFile(path.join(dist, "index.html"), "old HTML");
  await fs.writeFile(
    path.join(dist, "assets", "lazy-old.js"),
    "old lazy chunk",
  );
  return { staging, dist };
}

test("publishing and rolling back keep chunks needed by both generations of tabs", async (t) => {
  const { staging, dist } = await fixture(t);
  await fs.writeFile(
    path.join(staging, "index.html"),
    '<script src="/assets/new.js"></script>',
  );
  await fs.writeFile(path.join(staging, "assets", "new.js"), "new chunk");
  await publishFrontend(staging, dist);
  assert.equal(
    await fs.readFile(path.join(dist, "assets", "lazy-old.js"), "utf8"),
    "old lazy chunk",
  );
  assert.equal(
    await fs.readFile(path.join(dist, "assets", "new.js"), "utf8"),
    "new chunk",
  );
  assert.match(
    await fs.readFile(path.join(dist, "index.html"), "utf8"),
    /new.js/,
  );
  await fs.writeFile(path.join(staging, "index.html"), "old HTML");
  await publishFrontend(staging, dist);
  assert.equal(
    await fs.readFile(path.join(dist, "index.html"), "utf8"),
    "old HTML",
  );
  assert.equal(
    await fs.readFile(path.join(dist, "assets", "new.js"), "utf8"),
    "new chunk",
  );
});

test("an incomplete build leaves the served release intact", async (t) => {
  const { staging, dist } = await fixture(t);
  await fs.writeFile(path.join(staging, "assets", "new.js"), "new chunk");
  await assert.rejects(
    publishFrontend(staging, dist),
    /Incomplete frontend build/,
  );
  assert.equal(
    await fs.readFile(path.join(dist, "index.html"), "utf8"),
    "old HTML",
  );
  await assert.rejects(fs.access(path.join(dist, "assets", "new.js")));
});

test("an immutable asset collision fails before publishing any files", async (t) => {
  const { staging, dist } = await fixture(t);
  await fs.writeFile(path.join(staging, "index.html"), "new HTML");
  await fs.writeFile(path.join(staging, "assets", "new.js"), "new chunk");
  await fs.writeFile(
    path.join(staging, "assets", "lazy-old.js"),
    "changed bytes",
  );
  await assert.rejects(publishFrontend(staging, dist), /different contents/);
  assert.equal(
    await fs.readFile(path.join(dist, "index.html"), "utf8"),
    "old HTML",
  );
  assert.equal(
    await fs.readFile(path.join(dist, "assets", "lazy-old.js"), "utf8"),
    "old lazy chunk",
  );
  await assert.rejects(fs.access(path.join(dist, "assets", "new.js")));
});

test("deployments delete retired assets after 90 days but protect still-used old files", async (t) => {
  const { staging, dist } = await fixture(t);
  await fs.writeFile(path.join(staging, "index.html"), "first HTML");
  await fs.writeFile(path.join(staging, "assets", "shared.js"), "shared chunk");
  await publishFrontend(staging, dist, { now: 0 });
  await fs.unlink(path.join(staging, "assets", "shared.js"));
  await fs.writeFile(path.join(staging, "assets", "active.js"), "active chunk");
  await publishFrontend(staging, dist, { now: 100 * DAY });
  await assert.rejects(fs.access(path.join(dist, "assets", "lazy-old.js")));
  // shared.js was current until day 100, so its grace period starts at retirement.
  await fs.access(path.join(dist, "assets", "shared.js"));
  await fs.utimes(
    path.join(dist, "assets", "active.js"),
    new Date(0),
    new Date(0),
  );
  await publishFrontend(staging, dist, { now: 190 * DAY });
  await fs.access(path.join(dist, "assets", "shared.js"));
  await publishFrontend(staging, dist, { now: 191 * DAY });
  await assert.rejects(fs.access(path.join(dist, "assets", "shared.js")));
  assert.equal(
    await fs.readFile(path.join(dist, "assets", "active.js"), "utf8"),
    "active chunk",
  );
});

test("unknown metadata versions fail without changing the served files", async (t) => {
  const { staging, dist } = await fixture(t);
  await fs.writeFile(path.join(staging, "index.html"), "new HTML");
  await fs.writeFile(path.join(staging, "assets", "new.js"), "new chunk");
  await fs.writeFile(
    `${dist}.asset-retention.json`,
    JSON.stringify({ format: "lensisku.frontend-assets", version: 2 }),
  );
  await assert.rejects(publishFrontend(staging, dist), /unsupported/);
  assert.equal(
    await fs.readFile(path.join(dist, "index.html"), "utf8"),
    "old HTML",
  );
  await fs.access(path.join(dist, "assets", "lazy-old.js"));
});

test("a deployment fails safely if another publisher holds the lock", async (t) => {
  const { staging, dist } = await fixture(t);
  await fs.mkdir(`${dist}.publish-lock`);
  await assert.rejects(publishFrontend(staging, dist), { code: "EEXIST" });
  assert.equal(
    await fs.readFile(path.join(dist, "index.html"), "utf8"),
    "old HTML",
  );
});
