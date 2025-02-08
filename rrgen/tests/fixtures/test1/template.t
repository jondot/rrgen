{% set something = 1 -%}
to: tests/fixtures/test1/generated/{{name}}.txt
injections:
- into: tests/fixtures/test1/generated/prepend.txt
  prepend: true
  content: "this was prepended"
- into: tests/fixtures/test1/generated/append.txt
  append: true
  content: "this was appended"
- into: tests/fixtures/test1/generated/skipped.txt
  skip_if: "be skipped"
  append: true
  content: "this was appended"
- into: tests/fixtures/test1/generated/before.txt
  content: "// doc comment"
  before: "pub class"
- into: tests/fixtures/test1/generated/before_inline.txt
  content: "Hello"
  before: "World"
  inline: true
- into: tests/fixtures/test1/generated/before_last.txt
  content: "before-last"
  before_last: "\\]"
- into: tests/fixtures/test1/generated/before_last_inline.txt
  content: "Hello"
  before_last: "World"
  inline: true
- into: tests/fixtures/test1/generated/before_all_inline.txt
  content: "Hello"
  before_all: "World"
  inline: true
- into: tests/fixtures/test1/generated/before_all.txt
  content: "//fields"
  before_all: "pub struct"
- into: tests/fixtures/test1/generated/after.txt
  content: "field: integer"
  after: "pub class"
- into: tests/fixtures/test1/generated/after_last.txt
  content: "field: integer"
  after_last: "\\{"
- into: tests/fixtures/test1/generated/after_last_inline.txt
  content: "World"
  inline: true
  after_last: "Hello"
- into: tests/fixtures/test1/generated/remove_lines.txt
  content: ""
  remove_lines: "Delete this line"
- into: tests/fixtures/test1/generated/after_inline.txt
  content: "World"
  after: "Hello"
  inline: true
- into: tests/fixtures/test1/generated/after_all_inline.txt
  content: "World"
  after_all: "Hello"
  inline: true
- into: tests/fixtures/test1/generated/after_all.txt
  content: "  //fields"
  after_all: "Hello"
- into: tests/fixtures/test1/generated/replace.txt
  content: "World"
  replace: "Hello"
- into: tests/fixtures/test1/generated/replace_all.txt
  content: "World"
  replace_all: "Hello"
---

hello, this is the file body.

variable: {{ name | pascal_case }}
