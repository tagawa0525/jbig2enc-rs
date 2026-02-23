# jbig2enc-rs

A Rust reimplementation of [jbig2enc](https://github.com/agl/jbig2enc), a JBIG2 encoder for bi-level (1 bpp) images.

## About jbig2enc

[jbig2enc](https://github.com/agl/jbig2enc) is a C++ encoder for [JBIG2](https://www.itu.int/rec/T-REC-T.88/en), a bi-level image compression format that achieves better compression than G4 through symbol extraction and reuse. It is widely used for embedding scanned document images into PDFs.

This project reimplements jbig2enc's design and algorithms in Rust, using [leptonica-rs](https://github.com/tagawa0525/leptonica-rs) as the image processing foundation. The original C++ source code serves as the primary reference, included as a git submodule under `reference/jbig2enc/`.

## Porting Status

Work in progress. The encoder is being ported incrementally following the structure of the original C++ implementation.

| Feature                       | Status      |
| ----------------------------- | ----------- |
| Generic region encoding       | Planned     |
| Symbol extraction             | Planned     |
| Symbol classification         | Planned     |
| Text region coding            | Planned     |
| Refinement coding             | Planned     |
| Multipage / PDF fragment mode | Planned     |

Details: `docs/plans/`

## Crate Structure

```text
jbig2enc-rs/
├── src/                   # Encoder implementation
├── docs/plans/            # Implementation plans
└── reference/
    ├── jbig2enc/          # Original C++ jbig2enc (porting reference)
    ├── leptonica/         # Original C Leptonica (API reference)
    └── leptonica-rs/      # Rust Leptonica (image processing foundation)
```

## Build & Test

```bash
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
```

### Fetching References

```bash
git submodule update --init --recursive
```

## Documentation

- `CLAUDE.md` — Development conventions and process rules
- `docs/plans/` — Implementation plans for each feature
- [leptonica-rs API compatibility notes](https://github.com/tagawa0525/leptonica-rs/blob/main/docs/porting/jbig2enc-api-compatibility.md) — leptonica-rs API compatibility notes (also available locally at `reference/leptonica-rs/docs/porting/jbig2enc-api-compatibility.md` after `git submodule update --init --recursive`)

## License

This project is distributed under the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0), the same license as the original [jbig2enc](https://github.com/agl/jbig2enc).

## How This Project Is Built

The porting work is carried out primarily by AI coding agents, including [Claude Code](https://docs.anthropic.com/en/docs/claude-code). A human maintainer defines the overall architecture, process rules, and acceptance criteria, while the agents read the original C++ source, write Rust code, and run tests under those constraints. Every commit goes through CI and automated review before merging.

## Acknowledgments

This project relies on the source code and design of the original C++ jbig2enc by Adam Langley. It also depends on [leptonica-rs](https://github.com/tagawa0525/leptonica-rs) as its image processing foundation, which in turn is a reimplementation of [Leptonica](http://www.leptonica.org/) by Dan Bloomberg.
