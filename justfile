test:
  cargo test
  cargo test --features=simd_json --no-default-features --test json_tree_test

demo:
  cargo run -p demo

web:
  cd demo && trunk serve --open

doc:
  cargo doc -p egui_json_tree --no-deps --all-features --open