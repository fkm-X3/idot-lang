# Matrix (Rust bootstrap)

This Rust crate is a bootstrap copy of the Idot build/test toolchain.
It mirrors the canonical implementations in `compiler/src/cmd_*.ido`.

## Making changes

Do **not** edit `matrix/src/commands.rs` directly. Instead:

1. Edit the Idot version in `compiler/src/cmd_*.ido`
2. Optionally sync the changes back here to keep the bootstrap working

The Idot version is the source of truth — the Rust version exists only so
that the self-hosted compiler can bootstrap itself.
