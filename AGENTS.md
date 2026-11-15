# AGENTS.md

Owner: `evento-globolo`  
Tracking: `DEN-1889`

Use focused pull requests, preserve interface compatibility, add tests with behavior changes, never commit credentials or customer data, and resolve conflicts semantically using both sides and relevant history.

## Repository-local Git worktrees

- Create or use a Git worktree only when the human operator explicitly authorizes it for the current task. Concurrency or a dirty checkout is not permission by itself.
- Put every authorized worktree at `<repository-root>/tmp/worktrees/<name>`; from the repository root, use `./tmp/worktrees/<name>`. Never place worktrees beside repositories or organization directories.
- Keep `tmp`, `temp`, `tmp/worktrees`, and `temp/worktrees` ignored in the repository-root `.gitignore`. Do not commit files from those directories.
- Relocate or remove a worktree only when the operator explicitly requests it. Before removal, preserve and publish intended changes, verify its commit is represented on the target branch, and confirm there are no tracked, untracked, ignored-sensitive, or in-use files that must survive. Remove it with `git worktree remove <path>` without `--force`; never delete a worktree directory with `rm`.

## Rust-first tests and scripts

Follow the [Evento Globolo organization-wide Rust-first policy](https://github.com/evento-globolo/.github/pull/37).
Write new first-party scripts, tests, contract audits, CI helpers and code generation in Rust (`cargo test`, `tests/*.rs`, or `cargo xtask`), **not Python**. Existing Python automation is legacy and must not be extended. Migrate it only after equivalent Rust checks, flags, error behavior, security gates, and exact-head CI are established. Do not remove or bypass a required check to make the policy pass. Python product code needs a narrowly documented application exception.
