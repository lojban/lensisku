-- Remove the now-unused best-definition materialization.
-- Export ranking is computed live by export_best_definitions(), and search
-- paths in src/jbovlaste/service.rs compute vote_scores directly from
-- definitionvotes. Keeping these triggers/tables/functions only adds
-- maintenance overhead and stale-data risk.

-- Drop the triggers that refreshed valsibestdefinitions.
DROP TRIGGER IF EXISTS on_definitionvotes_delete ON public.definitionvotes;
DROP TRIGGER IF EXISTS on_definitionvotes_insert ON public.definitionvotes;
DROP TRIGGER IF EXISTS on_definitionvotes_update ON public.definitionvotes;

-- Drop the helper functions.
DROP FUNCTION IF EXISTS public.refresh_valsibestdefinitions_for_delete();
DROP FUNCTION IF EXISTS public.refresh_valsibestdefinitions_for_upsert();
DROP FUNCTION IF EXISTS public.reload_valsibestdefinitions();
DROP FUNCTION IF EXISTS public.reset_valsibestdefinition(integer, integer);

-- Drop the materialized best-definition table and its dependent positive-only view.
DROP VIEW IF EXISTS public.valsibestguesses;
DROP TABLE IF EXISTS public.valsibestdefinitions;

-- Drop the positive-only filter view for natlang words; export now uses
-- natlangwordbestplaces directly so it can include non-positive scores when asked.
DROP VIEW IF EXISTS public.natlangwordbestguesses;
