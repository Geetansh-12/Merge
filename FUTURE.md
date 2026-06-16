# marked-rs Future Roadmap

While `marked-rs` already achieves 95.2% CommonMark spec compliance, 98.2% test suite success, and operates with zero unsafe blocks, there is always room to grow. This roadmap outlines the strategic direction for `marked-rs` post-hackathon.

## Phase 1: 100% Spec Compliance

The remaining 4.8% of compliance issues largely stem from esoteric edge cases in the CommonMark spec:
- Deeply nested tricky overlapping formatting (e.g. nested lists in blockquotes with lazy continuation lines)
- Complex soft-line break rules in the presence of trailing spaces
- Rare HTML block terminator conditions

**Goal:** Achieve 100% CommonMark 0.31.2 compliance without sacrificing our zero-unsafe guarantee.

## Phase 2: Plugin Ecosystem

`marked.js` is beloved for its massive plugin ecosystem. Our goal is to replicate this extensibility in Rust using a trait-based plugin system:

```rust
pub trait Plugin {
    fn hook_lexer(&self, src: &str) -> Option<Token>;
    fn hook_renderer(&self, token: &Token) -> Option<String>;
}
```

This will allow users to add custom syntax (like math equations or charts) trivially.

## Phase 3: WebAssembly Native Integration

With the WASM demo already functional, we plan to release official NPM packages leveraging WASM. This allows JS ecosystems to transparently swap out `marked.js` for `marked-rs` and gain a 3x speedup with zero code changes.

## Phase 4: Full Async Streaming API

The current implementation parses the entire string synchronously. For massive documents, an asynchronous token stream is ideal. 

We will introduce a `StreamLexer` that implements `futures::Stream<Item = Token>`, allowing rendering of markdown *as it downloads* over the network.
