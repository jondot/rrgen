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
    let rgen = RRgen::default();

    rgen.generate(
        &fs::read_to_string("tests/fixtures/test1/template.t").unwrap(),
        &vars,
    )
    .unwrap();
    assert!(!dir_diff::is_different(GENERATED, "tests/fixtures/test1/expected").unwrap());
}

#[test]
fn test_generate_multiple_headers() {
    let FROM = "tests/fixtures/multi_split/app";
    let GENERATED = "tests/fixtures/multi_split/generated";

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
    let rgen = RRgen::default();

    rgen.generate(
        &fs::read_to_string("tests/fixtures/multi_split/template.t").unwrap(),
        &vars,
    )
    .unwrap();
    assert!(!dir_diff::is_different(GENERATED, "tests/fixtures/multi_split/expected").unwrap());
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
    let rgen = RRgen::with_working_dir(&tree_fs.root);

    rgen.generate(
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
    let rgen = RRgen::default();

    rgen.generate(
        &fs::read_to_string("tests/fixtures/realistic/controller.t").unwrap(),
        &vars,
    )
    .unwrap();
    rgen.generate(
        &fs::read_to_string("tests/fixtures/realistic/task.t").unwrap(),
        &vars,
    )
    .unwrap();
    assert!(!dir_diff::is_different(GENERATED, "tests/fixtures/realistic/expected").unwrap());
}

#[cfg(test)]
mod template_tests{
    use serde_json::json;
    use rrgen::{GenResult, RRgen};

    #[test]
    fn test_run_template_by_name_minijinja() {
        let template_name = "test_template";
        let template_str = r#"
---
to: ./file.txt
message: "Hello"
---
Hello, {{ name }}!
"#;
        let rgen = RRgen::with_templates(vec![(template_name, template_str)]).unwrap();
        let vars = json!({ "name": "World" });
        let result = rgen.generate_by_template_with_name(template_name, &vars).unwrap();

        if let GenResult::Generated { message } = result {
            assert_eq!(message.unwrap(), "Hello");
        } else {
            panic!("Template generation failed");
        }
    }

    #[test]
    fn test_run_template_string_minijinja() {
        let rgen = RRgen::default();
        let vars = json!({ "name": "World" });
        let template_str = r#"
---
to: ./file.txt
message: "Hello"
---
Hello, {{ name }}!
"#;
        let result = rgen.generate(template_str, &vars).unwrap();

        if let GenResult::Generated { message } = result {
            assert_eq!(message.unwrap(), "Hello");
        } else {
            panic!("Template generation failed");
        }
    }
}