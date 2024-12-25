#![allow(non_snake_case)]
use std::fs;

use fs_extra::{self, dir::CopyOptions};
use rrgen::RRgen;
use serde_json::json;

#[test]
fn test_generate() {
    let FROM = "tests/fixtures/test1/app";
    let GENERATED = "tests/fixtures/test1/generated";

    let vars = json!({"name": "post"});
    fs_extra::dir::remove(GENERATED).unwrap();
    fs_extra::dir::copy(
        FROM,
        GENERATED,
        &CopyOptions {
            copy_inside: true,
            ..Default::default()
        },
    )
    .unwrap();
    let mut rrgen = RRgen::default();

    rrgen.generate(
        &fs::read_to_string("tests/fixtures/test1/template.t").unwrap(),
        &vars,
    )
    .unwrap();
    assert!(!dir_diff::is_different(GENERATED, "tests/fixtures/test1/expected").unwrap());
}

#[test]
fn test_generate_with_working_dir() {
    let tree_fs = tree_fs::TreeBuilder::default()
        .drop(true)
        .create()
        .expect("create temp file");
    let FROM = "tests/fixtures/test1/app";
    let GENERATED = "tests/fixtures/test1/generated";

    let vars = json!({"name": "post"});
    fs_extra::dir::copy(
        FROM,
        tree_fs.root.join(GENERATED),
        &CopyOptions {
            copy_inside: true,
            ..Default::default()
        },
    )
    .unwrap();
    let mut rrgen = RRgen::with_working_dir(&tree_fs.root);

    rrgen.generate(
        &fs::read_to_string("tests/fixtures/test1/template.t").unwrap(),
        &vars,
    )
    .unwrap();
    assert!(!dir_diff::is_different(
        tree_fs.root.join(GENERATED),
        "tests/fixtures/test1/expected"
    )
    .unwrap());
}

#[test]
fn test_realistic() {
    let FROM = "tests/fixtures/realistic/app";
    let GENERATED = "tests/fixtures/realistic/generated";

    let vars = json!({"name": "email_stats"});
    fs_extra::dir::remove(GENERATED).unwrap();
    fs_extra::dir::copy(
        FROM,
        GENERATED,
        &CopyOptions {
            copy_inside: true,
            ..Default::default()
        },
    )
    .unwrap();
    let mut rrgen = RRgen::default();

    rrgen.generate(
        &fs::read_to_string("tests/fixtures/realistic/controller.t").unwrap(),
        &vars,
    )
    .unwrap();
    rrgen.generate(
        &fs::read_to_string("tests/fixtures/realistic/task.t").unwrap(),
        &vars,
    )
    .unwrap();
    assert!(!dir_diff::is_different(GENERATED, "tests/fixtures/realistic/expected").unwrap());
}
