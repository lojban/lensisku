-- Re-queue classical lujvo that were marked "checked" as self-canonical
-- (canonical_word = word) without flipping type. Early classifier runs could
-- fail reconstruct when DB rafsi maps shadowed vlazba built-ins, then stamp
-- canonical_word = word and leave typeid = 4 forever (e.g. rivyzu'e).
-- Clearing the sentinel lets the background drain reclassify with the fixed
-- builtin fallback.

UPDATE valsi
SET canonical_word = NULL
WHERE typeid = 4
  AND source_langid = 1
  AND canonical_word IS NOT DISTINCT FROM word;
