---
name: lojban-vocabulary
description: Find complete, verified vocabulary for Lojban translation and composition, and constrain searches to official gismu and cmavo when requested.
---

# Lojban vocabulary skill

Apply this skill when the user asks for official words, asks to combine existing words, or asks to translate a phrase. These modes can apply together. Preserve constraints from earlier messages until the user changes them.

## Official vocabulary

When the user asks for "only official words", "official gismu and cmavo", or an equivalent restriction, pass `official_only: true` in **every** `jbovlaste_semantic_search` call, including refinements and particle lookups. This filters the candidate pool to official Lojban gismu and cmavo before ranking. A word's spelling or presence in a sample phrase does not establish official status. Select only results with type `gismu` or `cmavo`; do not substitute experimental entries, lujvo, fu'ivla, cmevla, or phrase entries. Earlier unfiltered results are reusable only if their returned type verifies eligibility. If no eligible match exists, report the missing concept; do not silently relax the restriction.

## Combine existing words

When asked to express a meaning by combining existing words, **decompose before the first search**, even if a single dictionary word might exist. Identify the component relations, entities, properties, modifiers, and grammatical links; preserve ambiguity with alternative components when needed. Search each distinct component in the user's language in a batched `queries` array. Do not search only the original compound meaning and wait for failure before decomposing.

Inspect definitions and place structures for compatible components. Include any cmavo needed to connect them. Reuse already retrieved evidence and search missing components with different glosses. Submit definitions for **all** selected components and connecting particles through `jbovlaste_resolve_results`, rather than returning only a whole-word match or the most prominent roots. For compounding requests, include attested rafsi or decomposition references when useful; do not invent an unattested lujvo or rafsi.

## Phrase translation and sample phrases

Before searching, make a coverage checklist of the source meaning and target grammar: predicates, participants/pronouns, properties, quantities, tense/aspect, negation, direction/location, connectives, descriptors, abstractions, quotations, and necessary separators or terminators. Translate meanings and grammatical functions; do not force a separate Lojban word for each source-language token.

Read relevant sample phrases supplied by the user or bundled in the core reference. Use their attested patterns to identify **every** needed word, including particles, rather than treating a sample as evidence only for its content words. Split joined cmavo only when the reference or retrieved evidence supports that segmentation; verify the constituent particles separately. A user sample supplies lookup spellings and context, not verified definitions or official status.

Search all uncovered content concepts and grammatical functions. Exact valsi queries are allowed for words attested in samples or the core reference; each valsi gets its own query. Core reference glosses and examples can guide selection, but words without a database definition reference must be looked up before final submission. Batch up to 24 queries per call; for longer inventories continue in subsequent calls. Review results before searching remaining gaps or alternative glosses.

Before finishing, compare the selected words against the checklist. Do not stop after a few relevant hits if any required component or particle remains uncovered. Submit at least one definition reference for **each distinct selected word**, including pronouns, descriptors, tense, connectives, abstractions, and terminators. Add sample/example references where useful, but a whole example reference does not replace the individual word definitions. Keep alternatives needed for ambiguous meanings. If evidence remains missing after reasonable refinements, report the uncovered functions through `message` instead of presenting an incomplete set as sufficient for translation.

The harness renders validated references as the final vocabulary output. Do not invent a composed sentence inside `exact_text`; it must be copied verbatim from the referenced database field.
