#!/usr/bin/env node
/**
 * Vite wrapper: when Tailwind/PostCSS/Vite config (or `tailwind.*.mjs` brand
 * layers) change, kill and respawn Vite in a fresh Node process so ESM-cached
 * `addComponents` exports are not stale. Plain `vite` HMR cannot clear that cache.
 */
import { execFileSync, spawn } from 'node:child_process'
import { watch } from 'node:fs'
import net from 'node:net'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const frontendRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const PORT = Number(process.env.PORT || 5173)

const watchFiles = [
  'vite.config.ts',
  'vite.config.js',
  'tailwind.config.js',
  'tailwind.config.ts',
  'postcss.config.cjs',
  'postcss.config.js',
  'tailwind.lingo.mjs',
  'tailwind.aqua-buttons.mjs',
  'tailwind.flat-buttons.mjs',
].map((f) => path.join(frontendRoot, f))

let child = null
let restartTimer = null
let shuttingDown = false

function portFree(port) {
  return new Promise((resolve) => {
    const s = net.createServer()
    s.once('error', () => resolve(false))
    s.once('listening', () => {
      s.close(() => resolve(true))
    })
    s.listen(port, '127.0.0.1')
  })
}

async function waitPortFree(port, timeoutMs = 10000) {
  const start = Date.now()
  while (Date.now() - start < timeoutMs) {
    if (await portFree(port)) return true
    await new Promise((r) => setTimeout(r, 100))
  }
  return false
}

/** Last resort on Linux when the previous Vite child did not release the port. */
function forceFreePort(port) {
  try {
    execFileSync('fuser', ['-k', `${port}/tcp`], { stdio: 'ignore' })
  } catch {
    /* nothing listening or fuser unavailable */
  }
}

function killChild() {
  return new Promise((resolve) => {
    const prev = child
    child = null
    if (!prev) {
      resolve()
      return
    }
    let settled = false
    const done = () => {
      if (settled) return
      settled = true
      resolve()
    }
    prev.once('exit', done)
    try {
      prev.kill('SIGTERM')
    } catch {
      done()
      return
    }
    setTimeout(() => {
      try {
        if (prev.exitCode === null) prev.kill('SIGKILL')
      } catch {
        /* already gone */
      }
      setTimeout(done, 200)
    }, 1200)
  })
}

async function start() {
  await killChild()
  if (!(await waitPortFree(PORT, 3000))) {
    forceFreePort(PORT)
    await waitPortFree(PORT, 5000)
  }
  child = spawn(
    'pnpm',
    ['exec', 'vite', '--host', '--strictPort', '--port', String(PORT), ...process.argv.slice(2)],
    {
      cwd: frontendRoot,
      stdio: 'inherit',
      env: process.env,
    },
  )
  child.on('exit', (code, signal) => {
    child = null
    if (shuttingDown) {
      process.exit(code ?? (signal ? 1 : 0))
    }
  })
}

function scheduleRestart(reason) {
  if (shuttingDown) return
  clearTimeout(restartTimer)
  restartTimer = setTimeout(() => {
    void (async () => {
      console.log(`\n  ➜  ${reason}; restarting Vite (clears Tailwind ESM cache)…\n`)
      await killChild()
      await start()
    })()
  }, 200)
}

for (const file of watchFiles) {
  try {
    watch(file, () => {
      scheduleRestart(`config changed (${path.basename(file)})`)
    })
  } catch {
    // Optional config file may be absent.
  }
}

function shutdown() {
  if (shuttingDown) return
  shuttingDown = true
  clearTimeout(restartTimer)
  void killChild().then(() => process.exit(0))
}

process.on('SIGINT', shutdown)
process.on('SIGTERM', shutdown)

void start()
