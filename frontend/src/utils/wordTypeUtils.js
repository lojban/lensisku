/**
 * Returns Tailwind classes for word type badges
 * @param {string} type - The word type (gismu, cmavo, etc.)
 * @returns {string} Tailwind classes for the badge
 */
export const getTypeClass = (type) => {
  const classMap = {
    gismu: 'bg-green-100 text-green-800',
    cmavo: 'bg-blue-100 text-blue-800',
    lujvo: 'bg-yellow-100 text-yellow-800',
    "fu'ivla": 'bg-orange-100 text-orange-800',
    'experimental cmavo': 'bg-pink-100 text-pink-800',
    'experimental gismu': 'bg-pink-100 text-pink-800',
    'cmavo-compound': 'bg-indigo-100 text-indigo-800',
    definition: 'bg-blue-100 text-blue-800',
    etymology: 'bg-purple-100 text-purple-800',
    comment: 'bg-green-100 text-green-800',
    valsi: 'bg-yellow-100 text-yellow-800',
    message: 'bg-indigo-100 text-indigo-800',
  }

  return classMap[type] || 'bg-gray-100 text-gray-800'
}
