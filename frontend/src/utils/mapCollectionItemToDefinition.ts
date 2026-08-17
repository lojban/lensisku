/** Collection list/search row fields needed to render a DefinitionCard. */
export type CollectionSearchItem = {
  item_id: number
  definition_id?: number | null
  word?: string | null
  valsi_id?: number | null
  definition?: string | null
  notes?: string | null
  lang_id?: number | null
  free_content_front?: string | null
  free_content_back?: string | null
  has_front_image?: boolean
  has_back_image?: boolean
  has_sound?: boolean
  sound_url?: string | null
  canonical_form?: string | null
  username?: string | null
}

export type CollectionRef = {
  collection_id: number
  name: string
}

/** Shape consumed by DefinitionCard for both dictionary-backed and custom-text items. */
export type CollectionDefinitionCard = {
  definitionid?: number
  item_id: number
  valsiid?: number | null
  valsiword: string
  word: string
  definition: string
  langid?: number | null
  notes?: string | null
  free_content_front?: string | null
  free_content_back?: string | null
  has_front_image?: boolean
  has_back_image?: boolean
  has_sound?: boolean
  sound_url?: string | null
  canonical_form?: string | null
  username?: string | null
  collection_id: number
  collection_name: string
}

export function mapCollectionItemToDefinition(
  item: CollectionSearchItem,
  collection: CollectionRef
): CollectionDefinitionCard {
  const front = item.word ?? item.free_content_front ?? ''
  return {
    definitionid: item.definition_id || undefined,
    item_id: item.item_id,
    valsiid: item.valsi_id,
    valsiword: front,
    word: front,
    definition: item.definition ?? item.free_content_back ?? '',
    langid: item.lang_id,
    notes: item.notes,
    free_content_front: item.free_content_front,
    free_content_back: item.free_content_back,
    has_front_image: item.has_front_image,
    has_back_image: item.has_back_image,
    has_sound: item.has_sound,
    sound_url: item.sound_url,
    canonical_form: item.canonical_form,
    username: item.username,
    collection_id: collection.collection_id,
    collection_name: collection.name,
  }
}

export function collectionDefinitionCardKey(def: CollectionDefinitionCard): string {
  return `ci-${def.collection_id}-${def.item_id}`
}
