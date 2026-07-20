import { useI18n } from 'vue-i18n'
import i18n from '@/i18n'

const pad = (n: number) => String(n).padStart(2, '0')

function getByPath(obj: unknown, path: string): unknown {
  const keys = path.split('.')
  let current: unknown = obj
  for (const key of keys) {
    if (current == null || typeof current !== 'object') return undefined
    current = (current as Record<string, unknown>)[key]
  }
  return current
}

function getTemplate(locale: string, key: string): string {
  const current = getByPath(i18n.global.getLocaleMessage(locale), key)
  if (typeof current === 'string') return current

  const fallback = getByPath(i18n.global.getLocaleMessage('jbo'), key)
  if (typeof fallback === 'string') return fallback

  return key === 'date.time' ? '{hours}:{minutes}' : '{year}-{month}-{day} {hours}:{minutes}'
}

export function useDateFormat() {
  const { locale } = useI18n()

  const parts = (date: Date) => ({
    year: String(date.getFullYear()),
    month: pad(date.getMonth() + 1),
    day: pad(date.getDate()),
    hours: pad(date.getHours()),
    minutes: pad(date.getMinutes()),
  })

  const format = (value: Date | number | string | undefined, templateKey: string): string => {
    if (value === undefined || value === null) return ''

    const date =
      value instanceof Date ? value : new Date(typeof value === 'number' ? value * 1000 : value)

    if (Number.isNaN(date.getTime())) return ''

    const template = getTemplate(locale.value, templateKey)
    const p = parts(date)
    return template.replace(/\{(\w+)\}/g, (_, key) => p[key as keyof typeof p] ?? '')
  }

  return {
    formatDate: (value: Date | number | string) => format(value, 'date.date'),
    formatTime: (value: Date | number | string) => format(value, 'date.time'),
    formatDateTime: (value: Date | number | string) => format(value, 'date.dateTime'),
    format,
  }
}
