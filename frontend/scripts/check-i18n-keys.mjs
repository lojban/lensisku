#!/usr/bin/env node
/**
 * Finds vue-i18n keys referenced in source that are missing from locale JSON files.
 * Mirrors runtime warnings when a key is missing for the active locale (e.g. en, jbo, ru, ja).
 *
 * Usage: node scripts/check-i18n-keys.mjs
 *        pnpm run i18n:check
 */

import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const frontendRoot = path.join(__dirname, '..')

/** @param {Record<string, unknown>} obj */
function flattenKeys(obj, prefix = '') {
  /** @type {string[]} */
  const out = []
  for (const [k, v] of Object.entries(obj)) {
    const p = prefix ? `${prefix}.${k}` : k
    if (v !== null && typeof v === 'object' && !Array.isArray(v)) {
      out.push(...flattenKeys(/** @type {Record<string, unknown>} */ (v), p))
    } else {
      out.push(p)
    }
  }
  return out
}

/**
 * @param {string} dir
 * @param {string[]} exts
 * @param {string[]} acc
 */
function walkSource(dir, exts, acc = []) {
  for (const name of fs.readdirSync(dir)) {
    if (name === 'node_modules' || name === 'dist' || name === '.git') continue
    const full = path.join(dir, name)
    const st = fs.statSync(full)
    if (st.isDirectory()) walkSource(full, exts, acc)
    else if (exts.some((e) => name.endsWith(e))) acc.push(full)
  }
  return acc
}

/** @param {string} s */
function unescapeJsString(s) {
  return s.replace(/\\(.)/g, (_, c) => {
    if (c === 'n') return '\n'
    if (c === 'r') return '\r'
    if (c === 't') return '\t'
    return c
  })
}

/**
 * Static first-argument keys for t / $t / i18n.t(…).
 * Skips dynamic template literals containing ${…}.
 * @param {string} content
 * @returns {Map<string, Set<string>>} key -> file paths
 */
function collectReferencedKeys(content, filePath) {
  /** @type {Map<string, Set<string>>} */
  const keyToFiles = new Map()

  function add(key) {
    if (!key || key.includes('${')) return
    let set = keyToFiles.get(key)
    if (!set) {
      set = new Set()
      keyToFiles.set(key, set)
    }
    set.add(filePath)
  }

  const reSingle = /(?:\$t|i18n\.t|\bt)\s*\(\s*'((?:[^'\\]|\\.)*)'/gs
  const reDouble = /(?:\$t|i18n\.t|\bt)\s*\(\s*"((?:[^"\\]|\\.)*)"/gs
  const reTpl = /(?:\$t|i18n\.t|\bt)\s*\(\s*`([^`${}]*)`/gs

  let m
  while ((m = reSingle.exec(content))) add(unescapeJsString(m[1]))
  while ((m = reDouble.exec(content))) add(unescapeJsString(m[1]))
  while ((m = reTpl.exec(content))) add(unescapeJsString(m[1]))

  return keyToFiles
}

/**
 * @param {Record<string, unknown>} base
 * @param {Record<string, unknown>} overlay
 */
function mergedLocaleKeys(base, overlay) {
  const baseSet = new Set(flattenKeys(base))
  const overlayKeys = flattenKeys(overlay)
  for (const k of overlayKeys) baseSet.add(k)
  return baseSet
}

function main() {
  const localesDir = path.join(frontendRoot, 'src/locales')
  const enPath = path.join(localesDir, 'en.json')
  const en = JSON.parse(fs.readFileSync(enPath, 'utf8'))

  /** Same merge as `src/i18n.ts`: overlay locales spread over `en`. */
  const overlayLocales = [
    { code: 'jbo', file: 'jbo.json' },
    { code: 'ru', file: 'ru.json' },
    { code: 'ja', file: 'ja.json' },
  ]

  /** @type {{ code: string, label: string, keys: Set<string> }[]} */
  const localeChecks = [
    { code: 'en', label: 'en (base locale)', keys: new Set(flattenKeys(en)) },
    ...overlayLocales.map(({ code, file }) => ({
      code,
      label: `${code} (en ∪ ${file})`,
      keys: mergedLocaleKeys(en, JSON.parse(fs.readFileSync(path.join(localesDir, file), 'utf8'))),
    })),
  ]

  const srcRoot = path.join(frontendRoot, 'src')
  const files = walkSource(srcRoot, ['.vue', '.ts'])

  /** @type {Map<string, Set<string>>} */
  const allRefs = new Map()

  for (const file of files) {
    const content = fs.readFileSync(file, 'utf8')
    const rel = path.relative(frontendRoot, file)
    const map = collectReferencedKeys(content, rel)
    for (const [k, paths] of map) {
      let set = allRefs.get(k)
      if (!set) {
        set = new Set()
        allRefs.set(k, set)
      }
      for (const p of paths) set.add(p)
    }
  }

  const missingByLocale = localeChecks.map(({ code, label, keys }) => ({
    code,
    label,
    missing: [...allRefs.keys()].filter((k) => !keys.has(k)).sort(),
  }))

  for (const { label, code, missing } of missingByLocale) {
    if (missing.length === 0) {
      console.log(`${label}: no missing keys.\n`)
      continue
    }
    console.log(`${label}: ${missing.length} key(s) not in ${code} messages:\n`)
    for (const k of missing) {
      const files = [...(allRefs.get(k) ?? [])].sort()
      console.log(`  ${k}`)
      for (const f of files) console.log(`    ${f}`)
    }
    console.log()
  }

  const hasAnyMissing = missingByLocale.some((m) => m.missing.length > 0)
  const exitCode = hasAnyMissing ? 1 : 0
  if (exitCode !== 0) {
    const failed = missingByLocale
      .filter((m) => m.missing.length > 0)
      .map((m) => m.code)
      .join(', ')
    console.error(
      `i18n check failed for: ${failed}. Add keys to src/locales/en.json and/or the overlay locale files (jbo.json, ru.json, ja.json) as in src/i18n.ts.`,
    )
  }
  process.exit(exitCode)
}

main()
