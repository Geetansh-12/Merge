# Upstream Bug Report (markedjs/marked)

**Title:** [Bug]: Incorrect parsing of deeply nested emphasis across link boundaries (CommonMark examples 449-461)

**Description:**
While building a differential fuzzer against `marked` v13 to verify CommonMark 0.31.2 compliance, we discovered a persistent divergence in how `marked` handles emphasis delimiters that attempt to cross link label boundaries. 

According to CommonMark Rule 9, emphasis markers inside a link label that do not have a matching marker inside the same link label cannot match with markers outside the link. 

**Steps to Reproduce:**
Parse the following markdown (CommonMark Example 449):
```markdown
*[foo*](url)
```

**Expected Behavior (CommonMark Spec):**
The outer `*` and inner `*` cannot form an emphasis block because the inner `*` is trapped inside the link. It should render the outer `*` as text, and the link normally, like so:
```html
<p>*<a href="url">foo*</a></p>
```

**Actual Behavior (marked v13):**
`marked` allows the emphasis to incorrectly cross the link boundary, destroying the link structure:
```html
<p><em>[foo</em>](url)</p>
```

**Environment:**
- `marked` version: 13.0.0
- Node.js version: 20.x

We have reproduced this behavior identically in our Rust port, but it is a bug in the upstream `marked` logic for handling delimiter state across token boundaries.
