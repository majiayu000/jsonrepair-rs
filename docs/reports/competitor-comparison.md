# Competitor Comparison Report

Generated: `2026-06-02T16:48:45+00:00`
Corpus: `tests/fixtures/parity_cases.json`

## Adapters

| Adapter | Version / status |
| --- | --- |
| `jsonrepair-rs` | jsonrepair 0.2.1 |
| `js-jsonrepair` | 3.14.0 |
| `python-json-repair` | json-repair 0.55.1 |
| `llm-json` | skipped: llm_json not found on PATH |

## Summary

| Adapter | exact | semantic | different | error | skipped |
| --- | ---: | ---: | ---: | ---: | ---: |
| `jsonrepair-rs` | 36 | 0 | 0 | 0 | 0 |
| `js-jsonrepair` | 34 | 0 | 1 | 1 | 0 |
| `python-json-repair` | 10 | 9 | 17 | 0 | 0 |
| `llm-json` | 0 | 0 | 0 | 0 | 36 |

## Cases

| Case | Category | Source | `jsonrepair-rs` | `js-jsonrepair` | `python-json-repair` | `llm-json` |
| --- | --- | --- | --- | --- | --- | --- |
| `valid-object-pass-through` | `valid-json` | `upstream-representative` | exact | exact | semantic<br>valid JSON, formatting differs | skipped<br>llm_json not found on PATH |
| `single-quotes` | `quotes` | `upstream-representative` | exact | exact | exact | skipped<br>llm_json not found on PATH |
| `comments` | `comments` | `upstream-representative` | exact | exact | semantic<br>valid JSON, formatting differs | skipped<br>llm_json not found on PATH |
| `missing-comma-object` | `missing-commas` | `upstream-representative` | exact | exact | exact | skipped<br>llm_json not found on PATH |
| `missing-comma-array` | `missing-commas` | `upstream-representative` | exact | exact | exact | skipped<br>llm_json not found on PATH |
| `trailing-comma-object` | `trailing-commas` | `upstream-representative` | exact | exact | exact | skipped<br>llm_json not found on PATH |
| `truncated-object` | `truncation` | `upstream-representative` | exact | exact | exact | skipped<br>llm_json not found on PATH |
| `truncated-array` | `truncation` | `upstream-representative` | exact | exact | exact | skipped<br>llm_json not found on PATH |
| `truncated-string` | `truncation` | `upstream-representative` | exact | exact | different<br>target `"hello"`, got `<empty>` | skipped<br>llm_json not found on PATH |
| `jsonp-callback` | `jsonp` | `upstream-representative` | exact | exact | exact | skipped<br>llm_json not found on PATH |
| `mongodb-object-id` | `mongodb` | `upstream-representative` | exact | exact | different<br>target `{"_id": "123abc"}`, got `{"_id": "ObjectId(", "123abc": ""}` | skipped<br>llm_json not found on PATH |
| `mongodb-number-long` | `mongodb` | `upstream-representative` | exact | exact | different<br>target `{"count": "42"}`, got `{"count": "NumberLong(", "42": ""}` | skipped<br>llm_json not found on PATH |
| `ndjson-two-objects` | `ndjson` | `upstream-representative` | exact | exact | semantic<br>valid JSON, formatting differs | skipped<br>llm_json not found on PATH |
| `markdown-json-fence` | `llm-markdown` | `upstream-representative` | exact | exact | semantic<br>valid JSON, formatting differs | skipped<br>llm_json not found on PATH |
| `llm-fenced-python-keywords` | `llm-markdown` | `project-regression` | exact | exact | different<br>target `{"name": "Ada", "active": true, "notes": null}`, got `{"name": "Ada", "active": true, "notes": "None"}` | skipped<br>llm_json not found on PATH |
| `unquoted-url` | `urls` | `upstream-representative` | exact | exact | semantic<br>valid JSON, formatting differs | skipped<br>llm_json not found on PATH |
| `regex-token` | `regex-like` | `project-regression` | exact | exact | different<br>target `{"regex": "/standalone-styles.css/"}`, got `{"regex": ""}` | skipped<br>llm_json not found on PATH |
| `ellipsis-array` | `ellipsis` | `upstream-representative` | exact | exact | semantic<br>valid JSON, formatting differs | skipped<br>llm_json not found on PATH |
| `python-keywords` | `keywords` | `upstream-representative` | exact | exact | different<br>target `{"flag": true, "value": null}`, got `{"flag": true, "value": "None"}` | skipped<br>llm_json not found on PATH |
| `escaped-string-wrapper-balanced` | `escaped-json-strings` | `upstream-representative` | exact | exact | different<br>target `"hello world"`, got `<empty>` | skipped<br>llm_json not found on PATH |
| `escaped-string-wrapper-unbalanced` | `escaped-json-strings` | `upstream-representative` | exact | exact | different<br>target `"hello world"`, got `<empty>` | skipped<br>llm_json not found on PATH |
| `mongodb-isodate` | `mongodb` | `upstream-representative` | exact | exact | different<br>target `{"created": "2020-01-01"}`, got `{"created": "isodate(", "2020-01-01": ""}` | skipped<br>llm_json not found on PATH |
| `mongodb-number-decimal` | `mongodb` | `upstream-representative` | exact | exact | different<br>target `{"decimal": 4}`, got `{"decimal": "NumberDecimal(4)"}` | skipped<br>llm_json not found on PATH |
| `mongodb-empty-number-int` | `mongodb` | `project-regression` | exact | different<br>target `{"n": null}`, got `{"n": "new NumberInt()"}` | different<br>target `{"n": null}`, got `{"n": "new NumberInt()"}` | skipped<br>llm_json not found on PATH |
| `special-whitespace-en-space` | `special-whitespace` | `upstream-representative` | exact | exact | exact | skipped<br>llm_json not found on PATH |
| `special-whitespace-inside-string-preserved` | `special-whitespace` | `upstream-representative` | exact | exact | semantic<br>valid JSON, formatting differs | skipped<br>llm_json not found on PATH |
| `redundant-root-closers` | `redundant-closers` | `upstream-representative` | exact | exact | exact | skipped<br>llm_json not found on PATH |
| `redundant-array-object-mismatch` | `redundant-closers` | `upstream-representative` | exact | exact | exact | skipped<br>llm_json not found on PATH |
| `missing-colon-between-key-value` | `missing-separators` | `upstream-representative` | exact | exact | different<br>target `{"a": 1}`, got `{"a": ""}` | skipped<br>llm_json not found on PATH |
| `missing-value-object` | `missing-values` | `upstream-representative` | exact | exact | different<br>target `{"a":null}`, got `{"a": ""}` | skipped<br>llm_json not found on PATH |
| `missing-comma-newline-properties` | `missing-separators` | `upstream-representative` | exact | exact | semantic<br>valid JSON, formatting differs | skipped<br>llm_json not found on PATH |
| `signed-non-finite-object` | `non-finite-values` | `project-regression` | exact | error<br>Error: Unexpected character "+" at position 19 | different<br>target `{"x":null,"y":null}`, got `{"x": "Infinity", "y": "NaN"}` | skipped<br>llm_json not found on PATH |
| `root-value-list-scalars` | `root-value-lists` | `upstream-representative` | exact | exact | different<br>target `[ 1, 2 ]`, got `<empty>` | skipped<br>llm_json not found on PATH |
| `ndjson-comments-and-commas` | `ndjson` | `upstream-representative` | exact | exact | different<br>target `[ {}, {}, {} ]`, got `{}` | skipped<br>llm_json not found on PATH |
| `markdown-python-fence` | `llm-markdown` | `upstream-representative` | exact | exact | semantic<br>valid JSON, formatting differs | skipped<br>llm_json not found on PATH |
| `llm-prose-profile-object` | `llm-prose` | `project-regression` | exact | exact | different<br>target `[ "The model returned this object:", {"name": "Ada", "active": true, "skills": ["rust",...`, got `{"name": "Ada", "active": true, "skills": ["rust", "json"]}` | skipped<br>llm_json not found on PATH |

## Status Legend

- `exact`: output matches the corpus target string exactly.
- `semantic`: output parses to the same JSON value but uses different formatting.
- `different`: output differs semantically or cannot be parsed as JSON.
- `error`: adapter returned a non-zero exit or timed out.
- `skipped`: adapter was requested but its toolchain was unavailable.
