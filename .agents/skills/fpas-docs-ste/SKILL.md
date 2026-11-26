---
name: fpas-docs-ste
description: Write, edit, and review Markdown (.md) files in Functional Pascal using Simplified Technical English (STE) rules, an explicit avoid-list, and canonical FPAS, CLR, GUI, TUI, and MVU terminology.
---

# Functional Pascal Markdown style (STE)

Apply this skill to every `.md` file in this repository. Use clear, direct,
STE-inspired technical English and one canonical term for each concept. Follow
[AGENTS.md](../../../AGENTS.md) and [CONTRIBUTING.md](../../../CONTRIBUTING.md).

Load [terminology and avoid-list](./references/terminology.md) when drafting or
reviewing prose, UI pages, or API descriptions.

## Core STE rules

1. **Keep sentences short and focused.** Limit procedural sentences to 20 words
   and descriptive sentences to 25 words. Put one action in each step and one
   fact in each sentence.
2. **Use active voice and direct mood.** Name the actor that performs the work.
   Use imperative mood for steps and commands (`Run`, `Return`, `Set`). Use
   simple present tense for implemented behavior and contracts.
3. **Use exact modal verbs.** State guaranteed behavior directly (`returns`,
   `accepts`, `rejects`). Use `must` for mandatory rules and `can` for optional
   capabilities. Do not use `should` for requirements.
4. **Keep noun clusters short.** Limit noun clusters to three words unless
   naming an actual API, type, or command. Replace nominalizations with verbs
   (`the host runs the command`, not `command execution by the host`).
5. **Make references unambiguous.** Repeat the concrete noun when `it`, `this`,
   `that`, or `they` can refer to more than one item.
6. **Use one term per concept.** Follow the canonical terms and avoid-list in
   [terminology.md](./references/terminology.md). Do not swap synonyms such as
   `event` for `message` or `action` for `command`.
7. **State conditions before results.** Write `If validation fails, the host
   rejects the view.` instead of burying the condition at the end of a long
   sentence.
8. **Use parallel lists and tables.** Start every bullet in a procedural list
   with an imperative verb. Give table columns one consistent grammatical form.

## Repository Markdown contracts

- **Language and accuracy:** Write all documentation in English. Document only
  implemented behavior. Omit implementation history and backward-compatibility
  notes.
- **Single owner per rule:** Check [docs/README.md](../../../docs/README.md) for
  the owning page. Link to the owner instead of copying contracts or API tables.
- **Agent reference budgets:** Files under `docs/pascal/agents/` must stay
  within the `o200k_base` limits in
  [maintenance.md](../../../docs/pascal/agents/maintenance.md) (`<= 13,000`
  total, `README.md < 1,000`). Keep wording concise so repeating subjects does
  not waste tokens.
- **Policy file limits:** Keep [AGENTS.md](../../../AGENTS.md) to one Markdown
  link (`docs/pascal/agents/README.md`), with no fenced code blocks and no
  `dotnet ` or `cargo ` commands.
- **Topic structure and backlinks:** Start every handbook page under
  `docs/pascal/` with a `# ` heading. Include `## See also` in handbook
  articles outside `docs/pascal/agents/`. Keep `## Source files` backlinks
  current for first-party sources that point to the page.
- **Code fences and examples:** Follow
  [documentation-examples.md](../../../docs/pascal/tools/documentation-examples.md)
  for `pascal` fences (`fragment context=...`, `project=...`, `error=...`,
  `run stdout=...`). Use `sh` or `text` fences for commands. Use
  [comments.md](../../../docs/pascal/language/basics/comments.md) for FPAS
  source-attached Markdown comments.
- **UI documentation:** Keep [UI handbook](../../../docs/ui/README.md),
  [widget reference](../../../docs/ui/widgets.md), and
  [MVU runtime](../../../docs/ui/mvu/runtime.md) terminology consistent across
  GUI, TUI, and MVU pages.

## Authoring and review workflow

1. Identify the document's owner and verify whether the target page is under a
   token budget, example check, or backlink check.
2. Read [terminology.md](./references/terminology.md) for the relevant domain
   (FPAS/CLR, Effects/Workflows, MVU, GUI/CUI, TUI, or toolchain).
3. Draft or edit the Markdown using the core STE rules above. Keep existing
   anchors stable when other pages or source headers link to them.
4. Review the changed prose against the avoid-list:
   - Remove filler and hedging words (`simply`, `just`, `easily`, `basically`).
   - Split sentences longer than 20–25 words or containing multiple clauses.
   - Replace passive voice, nominalizations, and vague pronouns.
   - Check that every domain concept uses its single canonical term.
5. Verify the changes:
   - Run `git diff --check` to catch whitespace errors.
   - Check local links, anchors, and backlinks with `HandbookTests` and
     `SourceWikiTests` (see
     [development.md](../../../docs/build/development.md)).
   - Run `AgentReferenceTests` when touching `AGENTS.md` or
     `docs/pascal/agents/`.
   - Run `doc-examples` or `LanguageSpecificationTests` when changing `pascal`
     fences.
