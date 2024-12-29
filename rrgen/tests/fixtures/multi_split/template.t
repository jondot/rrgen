{% set something = 1 -%}
{% for i in [1,2] -%}
---
to: tests/fixtures/multi_split/generated/post{{i}}.txt
injections:
- into: tests/fixtures/multi_split/generated/prepend{{i}}.txt
  prepend: true
  content: "this was prepended{{i}}"
- into: tests/fixtures/multi_split/generated/append{{i}}.txt
  append: true
  content: "this was appended{{i}}"
- into: tests/fixtures/multi_split/generated/skipped{{i}}.txt
  skip_if: "be skipped"
  append: true
  content: "this was appended"
- into: tests/fixtures/multi_split/generated/before{{i}}.txt
  content: "// doc comment"
  before: "pub class"
- into: tests/fixtures/multi_split/generated/before_last{{i}}.txt
  content: "before-last"
  before_last: "\\]"
- into: tests/fixtures/multi_split/generated/after{{i}}.txt
  content: "field: integer{{i}}"
  after: "pub class"
- into: tests/fixtures/multi_split/generated/after_last{{i}}.txt
  content: "field: integer{{i}}"
  after_last: "\\{"
- into: tests/fixtures/multi_split/generated/remove_lines{{i}}.txt
  content: ""
  remove_lines: "Delete this line"
---
hello, this is the file body.

variable: {{ i }}
{% endfor -%}
