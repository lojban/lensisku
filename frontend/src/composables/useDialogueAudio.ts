import { onBeforeUnmount, ref } from 'vue'
import { getDialogueLineAudioBlob } from '@/api'

/**
 * Per-line dialogue clips: fetched lazily, cached as object URLs, played one at a time.
 * `version` busts the cache after the editor regenerates audio.
 */
export function useDialogueAudio(getItemId: () => number | null | undefined) {
  const urls = new Map<string, string>()
  const playingLine = ref<number | null>(null)
  let audio: HTMLAudioElement | null = null
  let seq = 0

  async function urlFor(line: number, version = 0): Promise<string | null> {
    const id = getItemId()
    if (id == null) return null
    const key = `${id}:${line}:${version}`
    const hit = urls.get(key)
    if (hit) return hit
    try {
      const res = await getDialogueLineAudioBlob(id, line)
      const url = URL.createObjectURL(res.data as Blob)
      urls.set(key, url)
      return url
    } catch {
      return null
    }
  }

  function stop() {
    seq += 1
    if (audio) {
      audio.pause()
      audio = null
    }
    playingLine.value = null
  }

  async function playClip(line: number, version: number, token: number): Promise<void> {
    const url = await urlFor(line, version)
    if (!url || token !== seq) return
    const el = new Audio(url)
    audio = el
    playingLine.value = line
    await new Promise<void>((resolve) => {
      const done = () => {
        if (audio === el) {
          audio = null
          playingLine.value = null
        }
        resolve()
      }
      el.onended = done
      el.onerror = done
      el.onpause = done
      el.play().catch(done)
    })
  }

  /** Resolves when the clip ends (or immediately when it is missing / blocked). */
  async function play(line: number, version = 0): Promise<void> {
    stop()
    await playClip(line, version, seq)
  }

  /** Plays clips back to back; any stop()/play() cancels the rest. */
  async function playAll(lines: number[], version = 0) {
    stop()
    const token = seq
    for (const l of lines) {
      if (token !== seq) return
      await playClip(l, version, token)
    }
  }

  function clear() {
    stop()
    for (const u of urls.values()) URL.revokeObjectURL(u)
    urls.clear()
  }

  onBeforeUnmount(clear)

  return { play, playAll, stop, clear, playingLine }
}
