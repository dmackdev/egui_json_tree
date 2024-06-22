test:
  cargo test
  cargo test --features=simd_json --no-default-features --test json_tree_test

demo:
  cargo run --example=demo 

doc:
  cargo doc --no-deps --all-features --open