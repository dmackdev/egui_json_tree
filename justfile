test:
  cargo test
  cargo test --features=simd_json --no-default-features --test image_snapshot_tests

update_snapshots:
  UPDATE_SNAPSHOTS=1 cargo test --test image_snapshot_tests
  cargo test --features=simd_json --no-default-features --test image_snapshot_tests

demo:
  cargo run -p demo

web:
  cd demo && trunk serve --open

doc:
  cargo doc -p egui_json_tree --no-deps --all-features --open