import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import test from 'node:test'
import vm from 'node:vm'

// Exercise the form's actual handler without a browser, account or database.
const component = readFileSync(
  new URL('../src/pages/UpsertDefinition.vue', import.meta.url),
  'utf8'
)
const handler = component.slice(
  component.indexOf('const doAnalyzeWord = async () => {'),
  component.indexOf('const performValidateMathJax = async () => {')
)
const english = JSON.parse(readFileSync(new URL('../src/locales/en.json', import.meta.url), 'utf8'))

function form(analyzeWord) {
  const errors = []
  const context = {
    analyzeWord,
    word: { value: 'zazko' },
    wordType: { value: 'gismu' },
    recommended: { value: 'old recommendation' },
    problems: { value: { regular: ['old clash'] } },
    lujvoDecomposition: { value: ['old source'] },
    lastAnalyzedWord: { value: '' },
    isAnalyzing: { value: false },
    clearError: () => {},
    showError: (message) => errors.push(message),
    t: (key) => english.upsertDefinition[key.split('.').at(-1)],
  }
  vm.createContext(context)
  vm.runInContext(`${handler}\nglobalThis.runAnalysis = doAnalyzeWord`, context)
  return { context, errors }
}

test('HTTP 400 invalid morphology shows spelling guidance and clears stale results', async () => {
  const { context, errors } = form(async () => {
    throw { response: { status: 400, data: { success: false, error: 'Failed to parse word' } } }
  })
  await context.runAnalysis()
  assert.equal(errors[0], english.upsertDefinition.analyzeError)
  assert.match(errors[0], /consonant clusters/)
  assert.equal(context.word.value, 'zazko')
  assert.equal(context.wordType.value, '')
  assert.equal(context.recommended.value, '')
  assert.equal(Object.keys(context.problems.value).length, 0)
  assert.equal(context.lujvoDecomposition.value.length, 0)
  assert.equal(context.isAnalyzing.value, false)
})

test('transport failures remain distinct from invalid morphology', async () => {
  const { context, errors } = form(async () => {
    throw new Error('offline')
  })
  await context.runAnalysis()
  assert.equal(errors[0], english.upsertDefinition.analyzeErrorGeneric)
  assert.equal(context.wordType.value, '')
  assert.equal(context.isAnalyzing.value, false)
})

test('valid gismu still sets the classification', async () => {
  const { context, errors } = form(async () => ({
    data: { success: true, word_type: 'gismu', text: 'zazgo' },
  }))
  await context.runAnalysis()
  assert.equal(context.wordType.value, 'gismu')
  assert.equal(context.word.value, 'zazgo')
  assert.equal(errors.length, 0)
})
