import { Crepe } from '@milkdown/crepe'

/** Fixed toolbar. The slash menu and block handles stay off. */
export const crepeToolbarFeatures = {
  [Crepe.Feature.TopBar]: true,
  [Crepe.Feature.BlockEdit]: false,
  [Crepe.Feature.Placeholder]: false,
}

function decodeHref(href: string): string {
  try {
    return decodeURIComponent(href)
  } catch {
    return href
  }
}

function looksLikeHost(value: string): boolean {
  return /^www\./i.test(value) || /^[^\s/]+\.[a-z]{2,}([:/]|$)/i.test(value)
}

const TOP_BAR_LINE = 44

/** Document icon for the raw-source toolbar control. */
export const sourceToolbarIcon = `
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6zm1 7V3.5L19.5 9H15zM8 12h8v1.5H8V12zm0 3h8v1.5H8V15z"/>
</svg>`

type SourceToolbarBuilder = {
  addGroup: (
    key: string,
    label: string
  ) => {
    addItem: (
      key: string,
      item: { icon: string; active: () => boolean; onRun: () => void }
    ) => unknown
  }
}

/** Put the Paragraph menu after every other toolbar group. */
export function placeHeadingLast(builder: { build: () => Array<{ key: string }> }) {
  const build = builder.build.bind(builder)
  builder.build = () => {
    const groups = build()
    const headingIndex = groups.findIndex((group) => group.key === 'heading')
    if (headingIndex < 0 || headingIndex === groups.length - 1) return groups
    const [heading] = groups.splice(headingIndex, 1)
    groups.push(heading)
    return groups
  }
}

/** Append a raw-source button after the built-in toolbar groups. */
export function addSourceToolbarItem(builder: SourceToolbarBuilder, onRun: () => void) {
  builder.addGroup('source', 'Source').addItem('raw-source', {
    icon: sourceToolbarIcon,
    active: () => false,
    onRun,
  })
}

const CHEVRON =
  '<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" aria-hidden="true"><path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" d="m6 9 6 6 6-6"/></svg>'

/** One toolbar row, with a chevron when the rest of the controls wrap. */
export function attachTopBarToggle(
  root: HTMLElement,
  labels: { expand: string; collapse: string; raw?: string }
): () => void {
  const bar = root.querySelector<HTMLElement>('.milkdown-top-bar')
  const inner = bar?.querySelector<HTMLElement>('.top-bar-inner')
  if (!bar || !inner || bar.querySelector('.milkdown-top-bar-toggle')) return () => {}

  bar.classList.add('milkdown-top-bar--collapsible')
  const sourceButton = [...inner.querySelectorAll<HTMLButtonElement>('.top-bar-item')].find(
    (item) => item.querySelector('path[d^="M14 2H6"]')
  )
  sourceButton?.classList.add('milkdown-top-bar-source')
  if (sourceButton && labels.raw) sourceButton.setAttribute('aria-label', labels.raw)
  const sourceDivider = sourceButton?.previousElementSibling?.classList.contains('top-bar-divider')
    ? sourceButton.previousElementSibling
    : null
  const sourceAnchor = document.createComment('')
  ;(sourceDivider ?? sourceButton)?.before(sourceAnchor)

  const button = document.createElement('button')
  button.type = 'button'
  button.className = 'milkdown-top-bar-toggle top-bar-item'
  button.innerHTML = CHEVRON
  /** Vue teleports Send into this host. Moving the host does not fight that teleport. */
  const actionHost = document.createElement('span')
  actionHost.className = 'milkdown-top-bar-action-host'
  let expanded = false
  let measuredWidth = -1
  let pinned = false

  const restoreSource = () => {
    if (!sourceAnchor.parentNode) return
    if (sourceDivider) sourceAnchor.after(sourceDivider)
    if (sourceButton) (sourceDivider ?? sourceAnchor).after(sourceButton)
  }

  const pinSource = () => {
    if (sourceDivider) bar.insertBefore(sourceDivider, button)
    if (sourceButton) bar.insertBefore(sourceButton, button)
  }

  /** First-row pins live on the bar. Expanded, they join the wrapping tools so Send is last. */
  const parkPins = (intoFlow: boolean) => {
    if (intoFlow) {
      restoreSource()
      inner.appendChild(button)
      inner.appendChild(actionHost)
      return
    }
    bar.appendChild(button)
    bar.appendChild(actionHost)
    if (pinned) pinSource()
  }

  const sync = () => {
    const width = Math.round(bar.getBoundingClientRect().width)
    if (width !== measuredWidth) {
      measuredWidth = width
      restoreSource()
      bar.appendChild(button)
      bar.appendChild(actionHost)
      pinned = inner.scrollHeight > TOP_BAR_LINE + 1
      if (!pinned) expanded = false
    }
    if (!pinned) expanded = false
    parkPins(pinned && expanded)
    button.hidden = !pinned
    bar.classList.toggle('milkdown-top-bar--overflow', pinned)
    bar.classList.toggle('is-expanded', expanded)
    button.setAttribute('aria-expanded', expanded ? 'true' : 'false')
    button.setAttribute('aria-label', expanded ? labels.collapse : labels.expand)
  }

  button.addEventListener('click', () => {
    expanded = !expanded
    sync()
  })
  bar.appendChild(button)
  bar.appendChild(actionHost)

  const observer = new ResizeObserver(sync)
  observer.observe(inner)
  sync()

  return () => {
    observer.disconnect()
    button.remove()
    actionHost.remove()
    bar.classList.remove(
      'milkdown-top-bar--collapsible',
      'milkdown-top-bar--overflow',
      'is-expanded'
    )
  }
}

/** Keep web addresses and site paths. A bare title becomes a wiki page link. */
export function resolveEditorHref(href: string): string {
  const value = decodeHref(href).trim()
  if (!value) return href
  if (/^[a-z][a-z0-9+.-]*:/i.test(value)) return value
  if (value.startsWith('/') || value.startsWith('#') || value.startsWith('?')) return value
  if (looksLikeHost(value)) return `https://${value}`
  const slug = encodeURIComponent(value.replace(/ /g, '_'))
  return `/wiki/${slug}`
}
