# Upstream jd v2.2.2 reference diffs

This directory captures example inputs and outputs produced by the official [`jd`](https://github.com/josephburnett/jd) CLI (v2.2.2 linux/amd64 build downloaded from the upstream release page).

Each subdirectory contains:

- **Input files** used as the first and second arguments to `jd` (JSON unless noted otherwise).
- **command.txt** – the exact command (relative paths) that was executed from within the folder to reproduce the output.
- **Diff/patch artifacts** written by `jd` for the scenario.

The examples exercise flags that affect diff rendering or patch application behaviour so they can be re-used for parity tests.

## Reproducing the data set

1. Download and make the upstream binary executable:
   ```bash
   curl -L -o jd https://github.com/josephburnett/jd/releases/download/v2.2.2/jd-amd64-linux
   chmod +x jd
   ```
2. Change into one of the scenario directories below and run the command recorded in `command.txt`.

All commands were executed with the working directory set to the scenario folder so the relative paths resolve correctly.

## Scenario overview

| Scenario | Key flags / modes | Notes |
| --- | --- | --- |
| `default-object` | *(none)* | Baseline object diff to capture native jd output. |
| `color-output` | `-color` | Demonstrates ANSI-coloured diff. |
| `format-patch` | `-f patch` | Emits RFC 6902 JSON Patch. |
| `format-merge` | `-f merge` | Emits RFC 7386 JSON Merge Patch. |
| `arrays-set` | `-set` | Treats arrays as sets (unordered, unique). |
| `arrays-multiset` | `-mset` | Treats arrays as multisets (bag semantics). |
| `arrays-setkeys` | `-setkeys id` | Object identity for set diffing by key. |
| `precision` | `-precision 0.001` | Floats within tolerance treated as equal. |
| `yaml` | `-yaml` | Parses YAML input and renders jd diff. |
| `translate-jd2patch` | `-t jd2patch` | Converts native jd diff to JSON Patch. |
| `translate-patch2jd` | `-t patch2jd` | Converts JSON Patch to native jd format. |
| `patch-mode` | `-p` | Applies jd diff to produce patched document. |
| `output-flag` | `-o diff.jd` | Uses built-in output redirection instead of shell `>`.

