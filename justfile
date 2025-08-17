test:
  cargo test --workspace
  cargo test --package egui_json_tree --test image_snapshot_tests --features simd_json --no-default-features

update_snapshots:
  UPDATE_SNAPSHOTS=1 cargo test --test image_snapshot_tests
  cargo test --package egui_json_tree --test image_snapshot_tests --features simd_json --no-default-features

demo:
  cargo run -p demo

web:
  cd demo && trunk serve --open

doc:
  cargo doc -p egui_json_tree --no-deps --all-features --open