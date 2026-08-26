/**
 * Transfer-direction icons (brandbook §5b).
 *
 * - **Import / upload** (data coming in, or a file sent to the server): arrow-up only
 *   (`Upload`, `FileUp`).
 * - **Export / download** (data leaving the app, or a file saved locally): arrow-down only
 *   (`Download`, `FileDown`).
 *
 * App code must import from here — not `Upload` / `Download` / `FileUp` / `FileDown` / `Import`
 * directly from `@lucide/vue` (enforced by ESLint). Lucide’s `Import` glyph is a down arrow and
 * must not be used for import actions.
 */
export { Upload as ImportIcon, Upload as UploadIcon, FileUp as FileImportIcon } from '@lucide/vue'
export {
  Download as DownloadIcon,
  Download as ExportIcon,
  FileDown as FileExportIcon,
} from '@lucide/vue'
