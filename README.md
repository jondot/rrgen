
# rrgen

A microframework for declarative code generation and injection.


## Getting started

Templates use `Tera` or `minijinja` as a templating language (similar to liquid), and use a special metadata/body separation with _frontmatter_.

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

---
***Creating multiple files***
---
You can use a single template that generates a body with multiple frontmatter separators. This is useful since you have a template
that will result in the creation of multiple files.

Example template controller.t:

```rust
{% for model in ['user','customer'] -%}
{% set name = model | snake_case -%}
---
to: src/controllers/{{ name }}.rs
injections:
- into: src/controllers/mod.rs
append: true
content: "pub mod {{ name }};"
- into: src/app.rs
after: "AppRoutes::"
content: "            .add_route(controllers::{{ name }}::routes())"
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
        .prefix("{{ name }}")
        .add("/", get(hello))
        .add("/echo", get(echo))
}
{% endfor -%}
```
Rendering the above template will create the following files:

- src/controllers/user.rs
- src/controllers/customer.rs

It will also inject into src/app.rs the following:
    
```rust
.add_route(controllers::user::routes())
.add_route(controllers::customer::routes())
```

With this approach, you can generate multiple files with a single template. And not having to repeat rendering the template for each model.

### Switch template engine
To use `minijinja` instead of `Tera`, you must set the following in Cargo.toml 

`rrgen = { version="X.X.X", default-features = false, features = ["minijinja"] }`