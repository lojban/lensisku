/** Comment payload for `CommentItem` and activity lists that render comments. */
export type CommentItemApiComment = {
  comment_id: number
  content: Array<{ type: string; data?: string }>
  reactions: Array<{ reaction: string; count: number; reacted: boolean; isNew?: boolean }>
  username: string
  time: number
  comment_num?: number
  parent_id?: number | null
  thread_id?: number
  definition?: string | null
  definition_id?: number | null
  valsi_id?: number | null
  valsi_word?: string | null
  parent_content?: Array<{ type: string; data?: string }>
  is_bookmarked?: boolean
  total_replies?: number
  natlang_word_id?: number
}
