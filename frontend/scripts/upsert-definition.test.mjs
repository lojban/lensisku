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
    analysisError: { value: 'previous error' },
    wordType: { value: 'gismu' },
    recommended: { value: 'old recommendation' },
    problems: { value: { regular: ['old clash'] } },
    lujvoDecomposition: { value: ['old source'] },
    lastAnalyzedWord: { value: '' },
    isAnalyzing: { value: false },
    clearError: () => {},
    showError: (message) => errors.push(message),
    t: (key, params = {}) =>
      english.upsertDefinition[key.split('.').at(-1)].replace(
        /\{(\w+)\}/g,
        (_, name) => params[name]
      ),
  }
  vm.createContext(context)
  const formatter = component
    .slice(component.indexOf('function formatDefinitionError('), component.indexOf('// Form state'))
    .replace('apiError: unknown): string', 'apiError)')
  vm.runInContext(`${formatter}\n${handler}\nglobalThis.runAnalysis = doAnalyzeWord`, context)
  return { context, errors }
}

test('HTTP 400 shows the precise server diagnostic inline, without a popup', async () => {
  const diagnostic = english.upsertDefinition.forbiddenConsonantPair
    .replace('{pair}', 'zk')
    .replace('{start}', '3')
    .replace('{end}', '4')
  const { context, errors } = form(async () => {
    throw {
      response: { status: 400, data: { success: false, error: 'FORBIDDEN_CONSONANT_PAIR|zk|3|4' } },
    }
  })
  await context.runAnalysis()
  assert.equal(context.analysisError.value, diagnostic)
  assert.equal(errors.length, 0)
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
  assert.equal(context.analysisError.value, english.upsertDefinition.analyzeErrorGeneric)
  assert.equal(errors.length, 0)
  assert.equal(context.wordType.value, '')
  assert.equal(context.isAnalyzing.value, false)
})

test('valid gismu still sets the classification', async () => {
  const { context, errors } = form(async () => ({
    data: { success: true, word_type: 'gismu', text: 'zazgo' },
  }))
  await context.runAnalysis()
  assert.equal(context.analysisError.value, '')
  assert.equal(context.wordType.value, 'gismu')
  assert.equal(context.word.value, 'zazgo')
  assert.equal(errors.length, 0)
})
