import { randomUUID } from "node:crypto";
import { promises as fs } from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

async function filesIn(directory, prefix = "") {
  const files = [];
  for (const entry of await fs.readdir(directory, { withFileTypes: true })) {
    const relative = path.join(prefix, entry.name);
    if (entry.isDirectory()) {
      files.push(
        ...(await filesIn(path.join(directory, entry.name), relative)),
      );
    } else if (entry.isFile()) {
      files.push(relative);
    } else {
      throw new Error(`Unsupported build entry: ${relative}`);
    }
  }
  return files;
}

const RETENTION_MS = 90 * 24 * 60 * 60 * 1000;

async function retentionState(destination, assets, now) {
  let previous;
  try {
    previous = JSON.parse(
      await fs.readFile(`${destination}.asset-retention.json`, "utf8"),
    );
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
  if (
    previous &&
    (previous.format !== "lensisku.frontend-assets" ||
      previous.version !== 1 ||
      !Array.isArray(previous.currentAssets) ||
      !previous.currentAssets.every((file) => typeof file === "string") ||
      !previous.lastReferencedAt ||
      typeof previous.lastReferencedAt !== "object" ||
      !Object.values(previous.lastReferencedAt).every(
        (time) => Number.isFinite(time) && time >= 0,
      ))
  ) {
    throw new Error("Invalid or unsupported frontend asset retention metadata");
  }
  let existing = [];
  try {
    existing = (await filesIn(path.join(destination, "assets"))).map((file) =>
      path.join("assets", file),
    );
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
  // Unknown legacy assets receive a full grace period. File mtimes do not tell us
  // whether the old HTML was still serving them just before this deployment.
  const lastReferencedAt = Object.fromEntries(
    existing.map((file) => [file, previous?.lastReferencedAt[file] ?? now]),
  );
  for (const file of [...(previous?.currentAssets ?? []), ...assets]) {
    lastReferencedAt[file] = now;
  }
  return {
    existing,
    metadata: {
      format: "lensisku.frontend-assets",
      version: 1,
      currentAssets: assets,
      lastReferencedAt,
    },
  };
}

async function publishUnlocked(staging, destination, now) {
  staging = path.resolve(staging);
  destination = path.resolve(destination);
  if (
    staging === destination ||
    staging.startsWith(`${destination}${path.sep}`)
  ) {
    throw new Error("Build staging must be outside the served directory");
  }
  const files = await filesIn(staging);
  const isAsset = (file) => file.startsWith(`assets${path.sep}`);
  if (!files.includes("index.html") || !files.some(isAsset)) {
    throw new Error(
      "Incomplete frontend build: index.html and assets are required",
    );
  }
  const assets = files.filter(isAsset);
  const { existing, metadata } = await retentionState(destination, assets, now);

  // Validate all existing immutable names before publishing anything. A changed file
  // must have a new hashed name; old tabs must keep receiving the original bytes.
  const existingAssets = new Set();
  for (const file of files.filter(isAsset)) {
    let existing;
    try {
      existing = await fs.readFile(path.join(destination, file));
    } catch (error) {
      if (error.code === "ENOENT") continue;
      throw error;
    }
    if (!existing.equals(await fs.readFile(path.join(staging, file)))) {
      throw new Error(
        `Asset name already exists with different contents: ${file}`,
      );
    }
    existingAssets.add(file);
  }

  // Assets first, other public files next, HTML last. Cleanup only follows success.
  const priority = (file) =>
    isAsset(file) ? 0 : file.endsWith(".html") ? 2 : 1;
  files.sort((a, b) => priority(a) - priority(b));
  for (const file of files) {
    if (existingAssets.has(file)) continue;
    const target = path.join(destination, file);
    await fs.mkdir(path.dirname(target), { recursive: true });
    const temporary = `${target}.publish-${randomUUID()}`;
    try {
      await fs.copyFile(path.join(staging, file), temporary);
      await fs.rename(temporary, target);
    } finally {
      await fs.rm(temporary, { force: true });
    }
  }
  const statePath = `${destination}.asset-retention.json`;
  const temporaryState = `${statePath}.${randomUUID()}`;
  try {
    await fs.writeFile(temporaryState, `${JSON.stringify(metadata)}\n`);
    await fs.rename(temporaryState, statePath);
  } finally {
    await fs.rm(temporaryState, { force: true });
  }
  const currentAssets = new Set(assets);
  let deleted = 0;
  for (const file of existing) {
    if (
      !currentAssets.has(file) &&
      metadata.lastReferencedAt[file] < now - RETENTION_MS
    ) {
      await fs.unlink(path.join(destination, file));
      deleted++;
    }
  }
  console.log(
    `Frontend published; removed ${deleted} assets unused for over 90 days`,
  );
}

export async function publishFrontend(
  staging,
  destination,
  { now = Date.now() } = {},
) {
  if (!Number.isFinite(now) || now < 0)
    throw new Error("Invalid deployment timestamp");
  destination = path.resolve(destination);
  const lock = `${destination}.publish-lock`;
  // Concurrent deployments must not clean up files another publisher is installing.
  await fs.mkdir(lock);
  try {
    await publishUnlocked(staging, destination, now);
  } finally {
    await fs.rmdir(lock);
  }
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  const [, , staging, destination] = process.argv;
  if (!staging || !destination) {
    throw new Error("Usage: node publish-frontend.mjs STAGING DIST");
  }
  await publishFrontend(staging, destination);
}
