# Changelog

## 0.6.0

### Breaking

- **An injection that cannot place its content is now an error.** An anchor
  regex (`before`, `before_last`, `after`, `after_last`) matching no line used to
  rewrite the target file unchanged and still report `injected: <file>`; an
  injection with no placement at all printed `warning: no injection made` and
  carried on reporting success. Both now return an error naming the file, the
  pattern, and the content that was dropped.

  This is the failure behind generated code that compiles and is never reached —
  a migration that never runs, a route that is never mounted — with nothing in
  the output to say so.

  `remove_lines` still succeeds when it matches nothing: the file is already in
  the state the template asked for.

- **A failed generation writes nothing.** Injections are now resolved before any
  file is written, so a failure leaves the tree untouched and the generator can
  simply be run again. Previously the target file was written first, which meant
  a retry hit `skip_exists`/`skip_glob` and returned before reaching the
  injection that had failed.

- `Error` gained variants (`InjectionTargetMissing`, `InjectionAnchorNotFound`,
  `InjectionHasNoPlacement`, replacing stringly-typed messages) and is now
  `#[non_exhaustive]`.

### Fixed

- An injection no longer strips the trailing newline from the file it edits.
  `str::lines` drops it and rejoining never put it back, so every injected file
  ended up with `\ No newline at end of file` in the user's diff.
- `skip_glob` resolves against the working directory set by
  `RRgen::with_working_dir`, like every other path in a template. It used to
  glob the process's current directory, so the skip silently never fired.
