# Vendored crates

## esp-idf-svc

Patched copy of `esp-idf-svc` 0.51.0 used via `[patch.crates-io]` in `esp32/Cargo.toml`.

Changes in `src/bt/a2dp.rs`:

- Harden `source_data_handler` against null/invalid length during A2DP stop.
- `clear_source_data_callback` / `restore_source_data_callback` for safe disconnect.
