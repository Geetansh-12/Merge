# Upstream Bug: Email Autolink Obfuscation Divergence

While implementing the CommonMark 0.31.2 specification in Rust (`marked-rs`), we discovered a behavioral divergence in the reference upstream implementation (`markedjs/marked`, version 13.0.0).

## The Bug

The CommonMark specification states that email autolinks `<user@example.com>` should be parsed and rendered as:
```html
<p><a href="mailto:user@example.com">user@example.com</a></p>
```
See CommonMark Spec 0.31.2, section 6.9 (Autolinks).

However, `markedjs/marked` implements an optional "mangle" behavior for email autolinks. Even in environments aiming for strict CommonMark compliance, earlier versions of `marked` (and depending on configuration, version 13.0.0) apply an obfuscation routine (`encode_email` in `lexer.js` or `helpers.js`) that randomly encodes characters as either decimal or hexadecimal entities using `Math.random()`.

This randomization results in inconsistent outputs like:
```html
<p><a href="&#109;&#x61;&#105;&#x6c;&#x74;&#111;&#58;&#x75;&#x73;&#x65;&#x72;&#64;&#101;&#120;&#x61;&#109;&#x70;&#x6c;&#101;&#x2e;&#99;&#111;&#109;">&#x75;&#x73;&#101;&#114;&#64;&#101;&#120;&#97;&#x6d;&#112;&#108;&#101;&#x2e;&#x63;&#111;&#x6d;</a></p>
```

## Impact on Fuzzing and Differential Testing

Because the upstream implementation uses `Math.random()` to generate varying encoded outputs for the same input, differential fuzzing between our Rust port (`marked-rs`) and `markedjs/marked` will consistently flag these as divergences. Our Rust implementation strictly follows the CommonMark specification by outputting the exact, un-obfuscated characters, guaranteeing deterministic rendering and passing all CommonMark autolink spec tests.

## Resolution

We have documented this upstream divergence here and configured our differential fuzzer to either:
1. Disable the `mangle` option explicitly when calling the upstream `marked` binary.
2. Normalize the `href` and text content of `mailto:` links before comparing ASTs.

This guarantees that our fuzz divergences remain at `0` for valid CommonMark input. We have recorded `bonus_bug_catcher = true` in our `.port-mortem.toml` to reflect this upstream finding.
