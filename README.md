# jd-rs

Rust port of the Go **jd** (JSON diff & patch) tool — aiming for byte-for-byte output parity with `jd` v2.2.2.

## Repository structure

```

jd-rs/
├─ LICENSE                  # MIT (aligned with upstream)
├─ README.md                # Overview, install, quickstart, compatibility
├─ CHANGELOG.md
├─ CONTRIBUTING.md
├─ CODE_OF_CONDUCT.md
├─ SECURITY.md
├─ ADRs/
│  ├─ 0001-json-number-semantics.md
│  └─ 0002-array-diff-lcs-design.md
├─ crates/
│  ├─ jd-core/
│  │  ├─ Cargo.toml
│  │  ├─ src/
│  │  │  ├─ lib.rs               # pub API surface
│  │  │  ├─ node.rs              # Node model + serde interop
│  │  │  ├─ path.rs              # Path & PathElem
│  │  │  ├─ options.rs           # Options & PathOptions
│  │  │  ├─ diff.rs              # core diff (incl. LCS & context)
│  │  │  ├─ patch.rs             # apply/reverse
│  │  │  ├─ render_jd.rs         # v2 format render
│  │  │  ├─ parse_jd.rs          # v2 format parser
│  │  │  ├─ json_patch.rs        # RFC 6902 subset
│  │  │  ├─ merge_patch.rs       # RFC 7386
│  │  │  └─ error.rs             # thiserror types
│  │  └─ tests/
│  │     ├─ roundtrip.rs
│  │     ├─ translators.rs
│  │     └─ numbers.rs
│  ├─ jd-cli/
│  │  ├─ Cargo.toml
│  │  └─ src/main.rs            # clap flags, IO, exit codes
│  ├─ jd-fuzz/
│  │  ├─ fuzz_targets/
│  │  │  ├─ parse_jd.rs
│  │  │  ├─ diff_fuzz.rs
│  │  │  └─ patch_fuzz.rs
│  │  └─ Cargo.toml
│  └─ jd-benches/
│     ├─ Cargo.toml
│     └─ benches/
│        ├─ arrays.rs
│        ├─ kubernetes_yaml.rs
│        └─ big_objects.rs
├─ tests/
│  ├─ cli/
│  │  ├─ help_snapshot.t
│  │  ├─ diff_basic.t
│  │  ├─ translate_patch.t
│  │  └─ yaml_mode.t
│  └─ fixtures/
│     ├─ json/
│     │  ├─ a.json
│     │  └─ b.json
│     ├─ yaml/
│     │  ├─ dep_a.yaml
│     │  └─ dep_b.yaml
│     └─ golden/
│        ├─ jd/
│        ├─ patch/
│        └─ merge/
├─ scripts/
│  ├─ gen_golden_from_go.sh      # uses Go jd v2.2.2 to (re)generate goldens
│  ├─ parity_check.sh            # compare Rust vs Go outputs byte-for-byte
│  └─ bench_vs_go.sh             # run benches & summarize
└─ .github/workflows/
   ├─ ci.yml
   ├─ parity.yml
   └─ release.yml
```
