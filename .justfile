generate-catchable-rs:
    chmod 666 src/catchable/generated.rs
    nix run .#src-catchable-rs
    chmod 444 src/catchable/generated.rs
