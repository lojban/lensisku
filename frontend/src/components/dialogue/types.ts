/** Mirrors src/dialogue/parser.rs + service.rs JSON. */
export interface DialogueLine {
  index: number
  speaker: string
  gender: 'm' | 'f' | null
  voice_override: string | null
  text: string
  gloss: string | null
}

interface DialogueChoiceOption {
  text: string
  correct: boolean
}

export interface DialogueQuestion {
  index: number
  kind: 'fill' | 'bank' | 'choice'
  prompt: string
  before: string
  after: string
  answers: string[]
  options: DialogueChoiceOption[]
}

interface DialogueParseIssue {
  line: number
  message: string
}

export interface Dialogue {
  title: string | null
  lines: DialogueLine[]
  questions: DialogueQuestion[]
  errors: DialogueParseIssue[]
}

interface DialogueLineAudio {
  index: number
  voice: string
  has_audio: boolean
}

export interface DialogueChip {
  id: string
  text: string
}

export interface DialogueView {
  item_id: number | null
  collection_id: number | null
  is_owner: boolean
  dialogue: Dialogue
  voices: Record<string, string>
  lines: DialogueLineAudio[]
  bank_chips: { question_index: number; chips: DialogueChip[] }[]
  notes: string
}

export interface DialogueGrade {
  correct: boolean
  expected: string
  rating: number
}

export const DIALOGUE_VOICES = ['Martin', 'Bernd', 'Victoria', 'Eva'] as const
export const NARRATOR = 'narrator'

/** Same normalisation as the server grader (lowercase, drop . , ! ? ", collapse spaces). */
function normalizeDialogueAnswer(s: string): string {
  return s
    .toLowerCase()
    .replace(/[.,!?"]/g, ' ')
    .split(/\s+/)
    .filter(Boolean)
    .join(' ')
}

export function gradeDialogueLocally(q: DialogueQuestion, answer: string): DialogueGrade {
  const a = normalizeDialogueAnswer(answer)
  if (q.kind === 'choice') {
    const right = q.options.find((o) => o.correct)
    return {
      correct: q.options.some((o) => o.correct && normalizeDialogueAnswer(o.text) === a),
      expected: right?.text ?? '',
      rating: 0,
    }
  }
  return {
    correct: q.answers.some((x) => normalizeDialogueAnswer(x) === a),
    expected: q.answers[0] ?? '',
    rating: 0,
  }
}
