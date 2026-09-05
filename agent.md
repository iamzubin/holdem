# agent.md — CI agent governance

These rules apply when running as an automated agent in CI
(`.github/workflows/opencode.yml`, `.github/workflows/opencode-review.yml`).
The action auto-commits and pushes **any** dirty tree when the session ends —
including build output and stray edits — so treat a clean tree as a hard requirement.

## Commits and pushes

- Do NOT commit or push unless the triggering user comment explicitly asks for code changes to be committed, pushed, or opened as a PR.
- Questions, reviews, explanations, approvals, and triage are answer-only: reply in the thread, leave the working tree untouched.
- If code changes were requested but committing was not: leave changes uncommitted in the workspace and say so in the reply.

## Tree hygiene

- Never touch `dist/`, lockfiles, or generated files unless the task requires it.
- Verify with `git status --short` before finishing; the tree must contain only intended changes.
