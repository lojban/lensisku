/**
 * Maximum number of cards in one open (course-wide) study session: the classic
 * "Study now" button (`/collections/:id/flashcards/study`) and the Lingo top-bar Study
 * button (`/collections/:id/lingo/study` without `levelId`). Level lessons are not capped.
 */
export const STUDY_SESSION_MAX_CARDS = 30

/** Cards still allowed in a capped session after `shown` cards. */
export const studySessionRoom = (shown: number) => Math.max(0, STUDY_SESSION_MAX_CARDS - shown)
