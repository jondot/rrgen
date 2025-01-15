
# rrgen

A microframework for declarative code generation and injection.


## Getting started

Templates use `Tera` as a templating language (similar to liquid), and use a special metadata/body separation with _frontmatter_.

The first part of the template instructs what the template should do, and which `injections` it should perform.

The second part is the actual target file that's being generated.


Example template `controller.t`:

```rust
---
to: tests/fixtures/realistic/generated/controllers/{{name | snake_case }}.rs
injections:
- into: tests/fixtures/realistic/generated/controllers/mod.rs
  append: true
  content: "pub mod {{ name | snake_case }};"
- into: tests/fixtures/realistic/generated/app.rs
  after: "AppRoutes::"
  content: "            .add_route(controllers::{{ name | snake_case }}::routes())"
---
#![allow(clippy::unused_async)]
use axum::{extract::State, routing::get};
use rustyrails::{
    app::AppContext,
    controller::{format, Routes},
    Result,
};

pub async fn echo(req_body: String) -> String {
    req_body
}

pub async fn hello(State(ctx): State<AppContext>) -> Result<String> {
    // do something with context (database, etc)
    format::text("hello")
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("{{ name | snake_case }}")
        .add("/", get(hello))
        .add("/echo", get(echo))
}
```

Rendering a template will create one or more files, potentially inject into files, and is done like so:

```rust
use std::fs;
use rrgen::Rgen;
use serde_json::json;

let rrgen = RRgen::default();
let vars = json!({"name": "post"});

rrgen.generate(
    &fs::read_to_string("tests/fixtures/test1/template.t").unwrap(),
    &vars,
)
.unwrap();
```

`vars` will be variables that are exposed both for the _frontmatter_ part and the _body_ part.

## Injection Types

The `rrgen` microframework supports various types of injections to modify existing files or generate new content. Below are the supported injection types and their descriptions:

### Prepend

- **Description**: Adds content at the beginning of the target file.
- **Usage**:
```yaml
- into: path/to/target/file.txt
  prepend: true
  content: "Content to prepend"
```
    
### Append
- **Description**: Adds content at the end of the target file.
- **Usage**:
```yaml
- into: path/to/target/file.txt
  append: true
  content: "Content to append"
```

### Skip If
- **Description**: Skips the injection if the specified condition is met.
- **Usage**:
```yaml
- into: path/to/target/file.txt
  skip_if: "Condition to skip"
  append: true
  content: "Content to append"
```
  
### Before
- **Description**: Inserts content before a specified pattern in the target file.
- **Usage**:
```yaml
- into: path/to/target/file.txt
  content: "Content to insert"
  before: "Pattern to match"
  inline: true # Optional, inserts content inline before the pattern
```

### Before Last
- **Description**: Inserts content before the last occurrence of a specified pattern in the target file.
- **Usage**:
```yaml
- into: path/to/target/file.txt
  content: "Content to insert"
  before_last: "Pattern to match"
  inline: true # Optional, inserts content inline before the last occurrence of pattern
```
  
### Before All
- **Description**: Inserts content before all the occurrences of a specified pattern in the target file.
- **Usage**:
```yaml
- into: path/to/target/file.txt
  content: "Content to insert"
  before_last: "Pattern to match"
  inline: true # Optional, inserts content inline before the last occurrence of pattern
```
  
### After
- **Description**: Inserts content after a specified pattern in the target file.
- **Usage**:
```yaml
- into: path/to/target/file.txt
  content: "Content to insert"
  after: "Pattern to match"
  inline: true # Optional, inserts content inline after the pattern
```
  
### After Last
- **Description**: Inserts content after the last occurrence of a specified pattern in the target file.
- **Usage**:
```yaml
- into: path/to/target/file.txt
  content: "Content to insert"
  after_last: "Pattern to match"
  inline: true # Optional, inserts content inline after the last occurrence of the pattern
```

### After Last
- **Description**: Inserts content after the all occurrences of a specified pattern in the target file.
- **Usage**:
```yaml
- into: path/to/target/file.txt
  content: "Content to insert"
  after_last: "Pattern to match"
  inline: true # Optional, inserts content inline after the last occurrence of the pattern
```
  
### Remove Lines
- **Description**: Removes lines that match a specified pattern in the target file.
- **Usage**:
```yaml
- into: path/to/target/file.txt
  content: ""
  remove_lines: "Pattern to match"
```

### Replace
- **Description**: Replaces a specified pattern with the provided content in the target file.
- **Usage**:
```yaml
- into: path/to/target/file.txt
  content: "Content to replace"
  replace: "Pattern to match"
```

### Replace All
- **Description**: Replaces all occurrences of a specified pattern with the provided content in the target file.
- **Usage**:
```yaml
- into: path/to/target/file.txt
  content: "Content to replace"
  replace_all: "Pattern to match"
```