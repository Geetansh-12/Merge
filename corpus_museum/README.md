# Corpus Museum

Welcome to the `marked-rs` Corpus Museum. This directory contains a curated selection of some of the most diabolical, catastrophic, and historically devastating edge cases in Markdown parsing history.

Our differential fuzzing harness automatically generated many of these to test the robustness of our CommonMark implementation.

## Exhibits

### 1. The Quadratic Bracket Bomb
**File:** `01_quadratic_brackets.md`
**Description:** A sequence of 50,000 opening brackets `[`. In regex-based parsers, attempting to match a link label and backtracking 50,000 times results in an O(N²) freeze. `marked-rs` instantly rejects it due to the 999 character link label cap and linear-time delimiter stack parsing.

### 2. The Overlapping Emphasis Nightmare
**File:** `02_overlapping_emphasis.md`
**Description:** `*[**_*]*_` — Interwoven asterisks and underscores with link brackets. This fundamentally breaks regex tokenizers. `marked-rs` passes perfectly using a CommonMark §6.2 compliant delimiter stack that prioritizes innermost pairs.

### 3. The Entity Ambiguity
**File:** `03_entity_ambiguity.md`
**Description:** `&amp;amp;amp;amp;amp;` inside an autolink `<mailto:test@example.com?subject=a&amp;b>`. Browsers and parsers often disagree on when to unescape entities. We meticulously replicated the spec's exact HTML5 entity decoding rules.

### 4. The Lazy Continuation Trick
**File:** `04_lazy_list.md`
**Description:** A blockquote containing a list, where the list is lazily continued on the next line without the blockquote `>` prefix. Strict rules regarding block containment vs lazy continuations.

### 5. Task List Chaos
**File:** `05_task_list_fuzz.md`
**Description:** A bullet item followed by `[x]` but separated by mixed tabs, spaces, and HTML blocks. Tests the exact token boundaries for GFM task list parsing.

---

*Note: The actual raw files are kept in the `fuzz/` test suite, but their spirits live on here in the museum!*
