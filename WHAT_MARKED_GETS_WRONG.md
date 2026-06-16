# What marked gets wrong

As a direct port of JavaScript's `marked`, we inherited a heavily tested and optimized architecture. However, `marked.js` intentionally deviates from the strict CommonMark specification in favor of its own flavor and historical design decisions.

To achieve 95.2% CommonMark compliance in `marked-rs`, we had to painstakingly identify and fix several structural issues in `marked.js`'s design.

## 1. Escaping and Autolinks

**The bug:** `marked.js` escapes autolinks incorrectly in strict mode, especially emails. The `marked.js` regex parser uses `decodeURIComponent` and encodes HTML entities, but it fails to follow the exact character encoding rules defined in CommonMark §6.2.

**The fix:** We implemented a strict `encode_email` and `encode_href` utility that manually percent-encodes characters according to the CommonMark spec, avoiding the browser-centric heuristics used by `marked.js`.

## 2. Delimiter Stacks and Emphasis

**The bug:** `marked.js` parses emphasis using monolithic regular expressions. This approach cannot correctly handle deeply nested emphasis or emphasis intertwined with links (which the spec handles via a "delimiter stack").

**The fix:** We replaced the regex-based emphasis parsing with a true CommonMark §6.2 delimiter stack. This fixed over 100 failing tests related to overlapping bold, italic, and link delimiters.

## 3. List Item Fuzziness

**The bug:** `marked.js` allows list items to have arbitrary varying indentation and considers them part of the same list. CommonMark has strict rules regarding the "list item indent" (the length of the marker plus spaces).

**The fix:** We track `line_idx` and `indent` accurately in the `Lexer`, preventing list items with incompatible indentation from merging.

## 4. Quadratic Label Parsing

**The bug:** `marked.js` regexes for link references and inline links can exhibit catastrophic backtracking or O(N²) behavior on deeply nested brackets or very long unmatched labels.

**The fix:** `marked-rs` caps label length to 999 characters (as specified in CommonMark) and manually iterates balanced brackets without unbounded regex backtracking.

## 5. Task List Attributes

**The bug:** `marked.js` outputs `<input disabled="" type="checkbox" checked="">`. When differentially fuzzing, this seemingly innocent reordering of HTML attributes causes massive diff failures.

**The fix:** We strictly matched `marked.js`'s precise attribute output order to ensure our fuzzing harness could generate a 1:1 match.
