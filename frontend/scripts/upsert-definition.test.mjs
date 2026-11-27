import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import test from 'node:test'
import vm from 'node:vm'
import { isAxiosError } from 'axios'

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

for (const [page, start, end, errorField] of [
  ['UpsertDefinition', 'const submitValsi =', '// Watch for definition changes', 'analysisError'],
  ['UpsertDefinitionMarkdown', 'async function submitValsi(', '</script>', 'renameError'],
]) {
  test(`${page} reports rename denial without saving other fields`, async () => {
    const source = readFileSync(new URL(`../src/pages/${page}.vue`, import.meta.url), 'utf8')
    const submit = source.slice(source.indexOf(start), source.indexOf(end))
    const permission = source.slice(
      source.indexOf('const isHeadwordReadOnly ='),
      source.indexOf('const showRenameNotice =')
    )
    let updateCalls = 0
    const fields = {
      isValid: true,
      isSubmitting: false,
      canCreate: true,
      isEditMode: true,
      isAuthor: true,
      renameDenied: false,
      definitionError: '',
      word: 'newvalsi',
      originalWord: 'oldvalsi',
      definition: 'draft definition',
      langId: '2',
      editDefinitionId: 67878,
      glossKeywords: [],
      placeKeywords: [],
      notes: '',
      etymology: '',
      jargon: '',
      ownerOnly: false,
      imageData: null,
      removeImage: false,
      showRafsiField: false,
      showSelmahoField: false,
      showPlaceKeywordFields: false,
      [errorField]: '',
    }
    const context = vm.createContext({
      ...Object.fromEntries(Object.entries(fields).map(([name, value]) => [name, { value }])),
      crepe: null,
      isAxiosError,
      computed: (getter) => ({
        get value() {
          return getter()
        },
      }),
      clearError: () => {},
      performValidateMathJax: async () => {},
      renameDefinition: async () => {
        throw { isAxiosError: true, response: { status: 403 } }
      },
      updateValsi: async () => {
        updateCalls++
      },
      t: (key) => english.upsertDefinition[key.split('.').at(-1)],
    })
    vm.runInContext(
      `${permission}\n${submit.replace(': Record<string, unknown>', '')}\nglobalThis.submit = submitValsi\nglobalThis.headwordReadOnly = isHeadwordReadOnly`,
      context
    )
    context.isAuthor.value = false
    assert.equal(context.headwordReadOnly.value, true)
    context.isAuthor.value = true
    assert.equal(context.headwordReadOnly.value, false)
    context.canCreate.value = false
    assert.equal(context.headwordReadOnly.value, true)
    context.canCreate.value = true
    await context.submit()
    assert.equal(context[errorField].value, english.upsertDefinition.renameForbidden)
    assert.equal(context.isSubmitting.value, false)
    assert.equal(context.renameDenied.value, true)
    assert.equal(context.headwordReadOnly.value, true)
    assert.equal(context.word.value, 'oldvalsi')
    assert.equal(context.definition.value, 'draft definition')
    assert.equal(updateCalls, 0)
  })
}

test('pasting into a read-only headword leaves its value intact', async () => {
  const source = readFileSync(
    new URL('../src/components/DynamicInput.vue', import.meta.url),
    'utf8'
  )
  const pasteHandler = source.slice(
    source.indexOf('const handlePaste ='),
    source.indexOf('const handleClear =')
  )
  const context = vm.createContext({
    props: { readonly: true },
    inputValue: { value: 'oldvalsi' },
    emit: () => assert.fail('read-only input emitted a change'),
  })
  vm.runInContext(`${pasteHandler}\nglobalThis.paste = handlePaste`, context)
  await context.paste({
    preventDefault: () => assert.fail('read-only input intercepted the paste'),
  })
  assert.equal(context.inputValue.value, 'oldvalsi')
})
