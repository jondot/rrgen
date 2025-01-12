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
- into: tests/fixtures/test1/generated/before_last.txt
  content: "before-last"
  before_last: "\\]"
- into: tests/fixtures/test1/generated/after.txt
  content: "field: integer"
  after: "pub class"
- into: tests/fixtures/test1/generated/after_last.txt
  content: "field: integer"
  after_last: "\\{"
- into: tests/fixtures/test1/generated/remove_lines.txt
  content: ""
  remove_lines: "Delete this line"
- into: tests/fixtures/test1/generated/after_inline.txt
  content: "World"
  after: "Hello"
  inline: true
- into: tests/fixtures/test1/generated/replace.txt
  content: "World"
  replace: "Hello"
---

hello, this is the file body.

variable: {{ name | pascal_case }}
