-- Prefer the clean mirrored text for imported MediaWiki pages. Their native
-- definitions hold edit history for the same page, not a second search result.
-- Native-only pages remain eligible; deleting a mirror restores its eligible
-- native representation. Redirect mirrors suppress their native history stubs too.
(d.kind <> 'native_wiki' OR NOT EXISTS (
    SELECT 1 FROM definitions native JOIN wiki_articles mirror
        ON mirror.page_id::text = native.metadata->>'mw_page_id'
    WHERE native.definitionid = d.source_id
))
