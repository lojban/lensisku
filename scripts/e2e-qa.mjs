#!/usr/bin/env node

/**
 * End-to-end smoke test for a Lensisku deployment.
 *
 * Run with: node scripts/e2e-qa.mjs
 * Optional: LENSISKU_TARGETS="https://example.test,http://localhost:8051" node scripts/e2e-qa.mjs
 */

import { randomUUID } from 'node:crypto'
import readline from 'node:readline/promises'
import { stdin, stdout } from 'node:process'

const DEFAULT_TARGETS = [
  'https://lensisku-dev.lojban.org',
  'https://lensisku.lojban.org',
  'http://localhost:8051',
]
const targets = (process.env.LENSISKU_TARGETS ?? DEFAULT_TARGETS.join(','))
  .split(',')
  .map((target) => target.trim().replace(/\/$/, ''))
  .filter(Boolean)

function askHidden(prompt) {
  return new Promise((resolve) => {
    stdout.write(prompt)
    const wasRaw = Boolean(stdin.isTTY && stdin.isRaw)
    if (stdin.isTTY) stdin.setRawMode(true)
    let value = ''
    const onData = (chunk) => {
      for (const char of chunk.toString('utf8')) {
        if (char === '\u0003') {
          if (stdin.isTTY) stdin.setRawMode(wasRaw)
          stdin.off('data', onData)
          stdout.write('\n')
          process.exit(130)
        } else if (char === '\r' || char === '\n') {
          if (stdin.isTTY) stdin.setRawMode(wasRaw)
          stdin.off('data', onData)
          stdout.write('\n')
          resolve(value)
          return
        } else if (char === '\u007f' || char === '\b') {
          value = value.slice(0, -1)
        } else if (char >= ' ') {
          value += char
        }
      }
    }
    stdin.on('data', onData)
    stdin.resume()
  })
}

async function getCredentials() {
  const rl = readline.createInterface({ input: stdin, output: stdout })
  const username = (await rl.question('Username or email: ')).trim()
  rl.close()
  const password = await askHidden('Password: ')
  if (!username || !password) throw new Error('Username and password are required.')
  return { username, password }
}

async function request(base, path, { token, method = 'GET', body } = {}) {
  const response = await fetch(`${base}/api${path}`, {
    method,
    headers: {
      ...(body === undefined ? {} : { 'Content-Type': 'application/json' }),
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    ...(body === undefined ? {} : { body: JSON.stringify(body) }),
    signal: AbortSignal.timeout(30_000),
  })
  const text = await response.text()
  let data
  try {
    data = text ? JSON.parse(text) : null
  } catch {
    data = text
  }
  if (!response.ok) {
    throw new Error(`${method} ${path} returned ${response.status}: ${typeof data === 'string' ? data : JSON.stringify(data)}`)
  }
  return data
}

async function runTarget(base, credentials) {
  console.log(`\n=== ${base} ===`)
  const cleanup = []
  let token
  let failed = false
  const check = (label, fn) => fn().then(() => console.log(`  PASS ${label}`))

  try {
    const auth = await request(base, '/auth/login', {
      method: 'POST',
      body: { username_or_email: credentials.username, password: credentials.password },
    })
    token = auth.access_token
    if (!token) throw new Error('Login response did not contain access_token.')
    await check('login and authenticated profile', () => request(base, '/auth/profile', { token }))

    const marker = randomUUID().replaceAll('-', '').slice(0, 12)
    const definitionText = `E2E QA definition ${marker}`
    const definition = await request(base, '/jbovlaste/valsi', {
      method: 'POST', token,
      body: {
        // A known Lojban word keeps the new entry in the source_langid=1
        // dictionary that the detail endpoint reads; the unique gloss makes
        // this a distinct definition while keeping deletion isolated.
        word: 'coi',
        definition: definitionText,
        lang_id: 2,
        source_langid: 1,
      },
    })
    if (!definition.success || !definition.definition_id) {
      throw new Error(`Definition creation failed: ${JSON.stringify(definition)}`)
    }
    const definitionId = definition.definition_id
    cleanup.push(async () => {
      await request(base, `/jbovlaste/definition/${definitionId}`, { method: 'DELETE', token })
    })
    console.log(`  PASS add definition (${definitionId})`)

    const editedDefinition = `${definitionText} edited`
    const updated = await request(base, `/jbovlaste/valsi/${definitionId}`, {
      method: 'PUT', token,
      body: { lang_id: 2, definition: editedDefinition },
    })
    if (!updated.success) throw new Error(`Definition edit failed: ${JSON.stringify(updated)}`)
    const savedDefinition = await request(base, `/jbovlaste/definition/${definitionId}`)
    if (!JSON.stringify(savedDefinition).includes(editedDefinition)) {
      throw new Error('Edited definition text was not returned by the API.')
    }
    console.log('  PASS edit definition')

    const comment = await request(base, '/comments', {
      method: 'POST', token,
      body: {
        definition_id: definitionId,
        subject: `E2E QA comment ${marker}`,
        content: [{ type: 'text', data: `E2E QA comment ${marker}` }],
      },
    })
    const commentId = comment.comment_id ?? comment.commentid ?? comment.id
    if (!commentId) throw new Error(`Comment response did not contain an id: ${JSON.stringify(comment)}`)
    cleanup.push(async () => {
      await request(base, `/comments/${commentId}`, { method: 'DELETE', token })
    })
    console.log(`  PASS add comment (${commentId})`)

    const wikiTitle = `E2E QA ${marker}`
    const wikiBody = `# E2E QA ${marker}\n\nInitial test page.`
    const wiki = await request(base, '/jbovlaste/valsi', {
      method: 'POST', token,
      body: { word: wikiTitle, definition: wikiBody, lang_id: 2, is_wiki: true },
    })
    if (!wiki.success || !wiki.definition_id) throw new Error(`Wiki creation failed: ${JSON.stringify(wiki)}`)
    const wikiId = wiki.definition_id
    cleanup.push(async () => {
      await request(base, `/jbovlaste/definition/${wikiId}`, { method: 'DELETE', token })
    })
    const wikiEdit = `# E2E QA ${marker}\n\nEdited test page.`
    const wikiUpdated = await request(base, `/jbovlaste/valsi/${wikiId}`, {
      method: 'PUT', token,
      body: { lang_id: 2, definition: wikiEdit, is_wiki: true },
    })
    if (!wikiUpdated.success) throw new Error(`Wiki edit failed: ${JSON.stringify(wikiUpdated)}`)
    const savedWiki = await request(base, `/jbovlaste/valsi/${encodeURIComponent(wikiTitle)}/wiki`)
    if (!JSON.stringify(savedWiki).includes('Edited test page.')) {
      throw new Error('Edited wiki content was not returned by the API.')
    }
    console.log(`  PASS add and edit wiki page (${wikiTitle})`)
  } catch (error) {
    failed = true
    console.error(`  FAIL ${error.message}`)
  } finally {
    for (const remove of cleanup.reverse()) {
      try {
        await remove()
        console.log('  CLEANUP removed test data')
      } catch (error) {
        failed = true
        console.error(`  CLEANUP FAILED: ${error.message}`)
      }
    }
    if (token) {
      try {
        await request(base, '/auth/logout', { method: 'POST', token, body: {} })
        console.log('  PASS logout')
      } catch (error) {
        failed = true
        console.error(`  FAIL logout: ${error.message}`)
      }
    }
  }
  return !failed
}

if (!targets.length) throw new Error('No target deployments configured.')
const credentials = await getCredentials()
let failures = 0
for (const target of targets) {
  if (!(await runTarget(target, credentials))) failures += 1
}
console.log(`\n${targets.length - failures}/${targets.length} deployments passed.`)
process.exitCode = failures ? 1 : 0
