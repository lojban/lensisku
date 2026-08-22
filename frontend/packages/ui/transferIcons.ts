/**
 * Transfer-direction icons (brandbook §5b, `tailwind.config.js`).
 *
 * - **Upload / export** (data leaving the user or app): arrow-up only (`Upload`, `FileUp`).
 * - **Download / import** (data saved locally or brought in from a file): arrow-down only
 *   (`Download`, `FileDown`, `Import`).
 *
 * App code must import from here — not `Upload` / `Download` / `FileUp` / `FileDown` / `Import`
 * directly from `@lucide/vue` (enforced by ESLint).
 */
export { Upload as ExportIcon, Upload as UploadIcon, FileUp as FileExportIcon } from '@lucide/vue'
export {
  Download as DownloadIcon,
  FileDown as FileImportIcon,
  Import as ImportIcon,
} from '@lucide/vue'
