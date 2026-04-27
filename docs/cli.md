# Command Line Interface

The `jsonrepair` binary repairs malformed JSON-like text from stdin or a file
and writes valid JSON to stdout or an output file.

## Usage

```bash
jsonrepair [OPTIONS] [INPUT_FILE]
```

When `INPUT_FILE` is omitted, the CLI reads from stdin. Passing `-` as the input
file also reads from stdin, which is useful when a shell script wants to keep a
placeholder for the input position.

## Output Behavior

| Command shape | Input | Output |
| --- | --- | --- |
| `jsonrepair` | stdin | stdout |
| `jsonrepair -` | stdin | stdout |
| `jsonrepair broken.json` | `broken.json` | stdout |
| `jsonrepair broken.json --output repaired.json` | `broken.json` | `repaired.json` |
| `jsonrepair --output repaired.json` | stdin | `repaired.json` |
| `jsonrepair --output=repaired.json` | stdin | `repaired.json` |

When `--output` is used, repair is completed into an internal buffer before the
destination file is written. If repair fails, an existing output file is left
unchanged.

## Exit Codes

| Code | Meaning |
| ---: | --- |
| `0` | Success, including `--help` and `--version`. |
| `1` | Input IO error, repair error, or output write error. |
| `2` | Command-line usage error, such as an unknown option, a missing `--output` path, or more than one input file. |

Error details are written to stderr. Repaired JSON is written to stdout only
when `--output` is not used.

## Shell Examples

Repair stdin to stdout:

```bash
printf "{name: 'Ada', active: True}" | jsonrepair
```

Repair stdin to a file:

```bash
printf "{name: 'Ada'}" | jsonrepair --output repaired.json
```

Use `-` as an explicit stdin placeholder:

```bash
cat broken.json | jsonrepair - --output repaired.json
```

Handle usage errors separately from repair or IO errors:

```bash
if ! jsonrepair "$input" --output "$output"; then
  code=$?
  if [ "$code" -eq 2 ]; then
    echo "invalid jsonrepair command line" >&2
  else
    echo "jsonrepair could not repair or write the document" >&2
  fi
fi
```
