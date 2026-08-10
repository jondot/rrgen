//! An injection must never quietly do nothing.
//!
//! Until 0.6, an injection whose anchor pattern matched no line rewrote the
//! target file unchanged and still reported `injected: <file>`. The failure had
//! no output of its own, so it surfaced much later as broken generated code —
//! in Loco, a migration that was written to disk, compiled fine, and never ran,
//! ending as a 500 on the first insert.
//!
//! These tests pin the boundary between "the file is already how the template
//! wants it" (fine, silent) and "the template could not do what it said" (an
//! error that names the file, the pattern, and the content that was dropped).

use rrgen::{Error, RRgen};
use serde_json::json;

const MIGRATOR: &str = "\
pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}
";

/// A migrator that was hand-written, so it never had the anchor comment.
const MIGRATOR_WITHOUT_ANCHOR: &str = "\
pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220101_000001_users::Migration)]
    }
}
";

fn template(placement: &str) -> String {
    format!(
        "to: migration/src/m20240101_000000_posts.rs\n\
         injections:\n\
         - into: migration/src/lib.rs\n  \
         {placement}\n  \
         content: \"            Box::new(m20240101_000000_posts::Migration),\"\n\
         ---\n\
         // the migration body\n"
    )
}

fn tree(migrator: &str) -> tree_fs::Tree {
    tree_fs::TreeBuilder::default()
        .drop(true)
        .add("migration/src/lib.rs", migrator)
        .create()
        .expect("create tree")
}

#[test]
fn a_missing_anchor_is_an_error_for_every_placement() {
    for placement in [
        r#"before: "inject-above""#,
        r#"before_last: "inject-above""#,
        r#"after: "inject-above""#,
        r#"after_last: "inject-above""#,
    ] {
        let tree = tree(MIGRATOR_WITHOUT_ANCHOR);
        let result = RRgen::with_working_dir(&tree.root).generate(&template(placement), &json!({}));

        assert!(
            matches!(result, Err(Error::InjectionAnchorNotFound { .. })),
            "`{placement}` matched nothing and did not report it: {result:?}"
        );
    }
}

/// The error has to be actionable on its own: the reader is a user of a
/// generator, not of rrgen, and has no idea what an injection is.
#[test]
fn the_error_names_the_file_the_pattern_and_the_dropped_content() {
    let tree = tree(MIGRATOR_WITHOUT_ANCHOR);
    let err = RRgen::with_working_dir(&tree.root)
        .generate(&template(r#"before: "inject-above""#), &json!({}))
        .expect_err("a missing anchor is an error");

    let message = err.to_string();
    for expected in [
        "migration/src/lib.rs",
        "before",
        "inject-above",
        "Box::new(m20240101_000000_posts::Migration),",
    ] {
        assert!(
            message.contains(expected),
            "the error does not mention `{expected}`:\n{message}"
        );
    }
}

/// A failed injection must leave the tree untouched, so re-running the
/// generator after fixing the anchor actually does the work.
///
/// This is not cosmetic: generators guard against duplicate output with
/// `skip_exists`/`skip_glob` on the file they create. If a failed run left that
/// file behind, the retry would skip and return before reaching the injection —
/// the one path that would never repair itself.
#[test]
fn a_failed_injection_writes_nothing() {
    let tree = tree(MIGRATOR_WITHOUT_ANCHOR);
    let generated = tree.root.join("migration/src/m20240101_000000_posts.rs");

    RRgen::with_working_dir(&tree.root)
        .generate(&template(r#"before: "inject-above""#), &json!({}))
        .expect_err("a missing anchor is an error");

    assert!(
        !generated.exists(),
        "the generated file was left behind by a generation that failed"
    );
    assert_eq!(
        std::fs::read_to_string(tree.root.join("migration/src/lib.rs")).unwrap(),
        MIGRATOR_WITHOUT_ANCHOR,
        "the injection target was rewritten by a generation that failed"
    );
}

#[test]
fn an_anchor_that_matches_injects_and_keeps_the_trailing_newline() {
    let tree = tree(MIGRATOR);
    RRgen::with_working_dir(&tree.root)
        .generate(&template(r#"before: "inject-above""#), &json!({}))
        .expect("the anchor is present");

    let migrator = std::fs::read_to_string(tree.root.join("migration/src/lib.rs")).unwrap();
    assert!(
        migrator.contains("Box::new(m20240101_000000_posts::Migration),"),
        "the migration was not registered:\n{migrator}"
    );
    assert!(
        migrator.ends_with("}\n"),
        "the injection stripped the file's trailing newline, which shows up in \
         the user's diff as `\\ No newline at end of file`:\n{migrator:?}"
    );
}

/// Two injections into one file — an import line and a registration line — is
/// the ordinary shape of a scaffold. Both must survive.
#[test]
fn injections_into_the_same_file_compose() {
    let tree = tree(MIGRATOR);
    let template = "to: out.txt\n\
        injections:\n\
        - into: migration/src/lib.rs\n  \
        before: \"inject-above\"\n  \
        content: \"            FIRST\"\n\
        - into: migration/src/lib.rs\n  \
        before: \"inject-above\"\n  \
        content: \"            SECOND\"\n\
        ---\n\
        body\n";

    RRgen::with_working_dir(&tree.root)
        .generate(template, &json!({}))
        .expect("both anchors are present");

    let migrator = std::fs::read_to_string(tree.root.join("migration/src/lib.rs")).unwrap();
    assert!(
        migrator.contains("FIRST") && migrator.contains("SECOND"),
        "one injection overwrote the other:\n{migrator}"
    );
}

/// An injection with content but no placement is a broken template, not a
/// no-op. It used to print `warning: no injection made` into stdout and carry
/// on reporting success.
#[test]
fn an_injection_with_no_placement_is_an_error() {
    let tree = tree(MIGRATOR);
    let template = "to: out.txt\n\
        injections:\n\
        - into: migration/src/lib.rs\n  \
        content: \"nowhere in particular\"\n\
        ---\n\
        body\n";

    let result = RRgen::with_working_dir(&tree.root).generate(template, &json!({}));
    assert!(
        matches!(result, Err(Error::InjectionHasNoPlacement { .. })),
        "expected a placement error, got {result:?}"
    );
}

/// Removal is the one strategy where matching nothing is a legitimate outcome:
/// the file is already in the state the template asked for. It must stay silent
/// *and* leave the file alone.
#[test]
fn a_removal_that_matches_nothing_is_not_an_error() {
    let tree = tree(MIGRATOR);
    let template = "to: out.txt\n\
        injections:\n\
        - into: migration/src/lib.rs\n  \
        remove_lines: \"this line is not in the file\"\n  \
        content: \"\"\n\
        ---\n\
        body\n";

    RRgen::with_working_dir(&tree.root)
        .generate(template, &json!({}))
        .expect("removing nothing is not a failure");

    assert_eq!(
        std::fs::read_to_string(tree.root.join("migration/src/lib.rs")).unwrap(),
        MIGRATOR,
        "a removal that matched nothing still rewrote the file"
    );
}

/// `skip_glob` is how a generator says "this component already exists". It is
/// declared as a repo-relative pattern like every other path in a template, so
/// it has to resolve against the working dir — globbing the process's current
/// directory instead means the skip silently never fires.
#[test]
fn skip_glob_resolves_against_the_working_dir() {
    let tree = tree(MIGRATOR);
    let generated = tree.root.join("migration/src/m20240101_000000_posts.rs");
    let template = |body: &str| {
        format!(
            "to: migration/src/m20240101_000000_posts.rs\n\
             skip_glob: \"migration/src/m????????_??????_posts.rs\"\n\
             ---\n\
             {body}\n"
        )
    };

    let rrgen = RRgen::with_working_dir(&tree.root);
    rrgen
        .generate(&template("the first body"), &json!({}))
        .expect("nothing matches the glob yet");

    let second = rrgen.generate(&template("a body that must not be written"), &json!({}));

    assert!(
        matches!(second, Ok(rrgen::GenResult::Skipped)),
        "skip_glob did not fire against the working dir: {second:?}"
    );
    assert_eq!(
        std::fs::read_to_string(&generated).unwrap(),
        "the first body\n",
        "the skipped generation wrote anyway"
    );
}
