// frontend/src/shims-js.d.ts
declare module '../api.js' {
  // You can add more specific types here later if needed
  // For now, 'any' will satisfy the compiler
  export function cancelBulkImport(processId: string): Promise<any>
  export function deleteBulkDefinitions(ids: string[]): Promise<any>
  export function getLanguages(): Promise<any>
  // Add other exports from api.js if necessary
}
