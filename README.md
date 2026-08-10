
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


## Injections

An injection edits a file that already exists. Every injection needs an `into:`
(the file), a `content:` (what to add), and exactly one **placement**:

| Placement      | Effect                                                          |
| -------------- | --------------------------------------------------------------- |
| `prepend: true`  | Put the content at the top of the file.                       |
| `append: true`   | Put the content at the bottom of the file.                    |
| `before:`        | Put it above the **first** line matching the regex.           |
| `before_last:`   | Put it above the **last** line matching the regex.            |
| `after:`         | Put it below the **first** line matching the regex.           |
| `after_last:`    | Put it below the **last** line matching the regex.            |
| `remove_lines:`  | Delete every line matching the regex.                         |

Two optional keys shape when an injection runs:

- `skip_if:` — a regex; if it matches anywhere in the target file, the injection
  is skipped. This is how you make generation idempotent.
- `into:` a file that does not exist is an error. Create it first.

### Injections either happen or fail

An anchor regex (`before`, `before_last`, `after`, `after_last`) that matches no
line is an **error**. So is an injection that declares `content` but no
placement.

This matters more than it sounds. A generator's job is to wire generated code
into the rest of the app — register a migration, mount a route, export a module.
When an anchor silently misses, the file is still written and still compiles; it
is simply never reached. The failure then surfaces days later as behaviour that
makes no sense, with nothing in the generator's output to point at.

When any injection fails, **nothing is written at all** — not the target file,
not the earlier injections. Fix the anchor and run the generator again.

`remove_lines` is the one exception: matching nothing means the file is already
in the state the template asked for, so it succeeds and leaves the file alone.

