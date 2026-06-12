#!/usr/bin/env python3
"""Download marked v13 test specs and produce JSON fixtures for integration tests."""
import json
import os
import urllib.request

BASE = "https://raw.githubusercontent.com/markedjs/marked/v13.0.0/test/specs"
OUT = "tests/original/specs/marked"


def fetch(url: str) -> str:
    with urllib.request.urlopen(url) as resp:
        return resp.read().decode("utf-8")


def fetch_json_specs(subdir: str, filenames: list[str], out_name: str) -> None:
    tests = []
    for name in filenames:
        url = f"{BASE}/{subdir}/{name}"
        print(f"Fetching {url}")
        data = json.loads(fetch(url))
        if isinstance(data, list):
            tests.extend(data)
        else:
            tests.append(data)
    out_path = os.path.join(OUT, out_name)
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(tests, f, indent=2)
    print(f"Wrote {out_path}: {len(tests)} tests")


def fetch_md_html_specs(subdir: str, pairs: list[str], out_name: str) -> None:
    tests = []
    for stem in pairs:
        md_url = f"{BASE}/{subdir}/{stem}.md"
        html_url = f"{BASE}/{subdir}/{stem}.html"
        print(f"Fetching {stem}")
        tests.append({
            "markdown": fetch(md_url),
            "html": fetch(html_url),
            "section": stem,
        })
    out_path = os.path.join(OUT, out_name)
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(tests, f, indent=2)
    print(f"Wrote {out_path}: {len(tests)} tests")


def main() -> None:
    os.makedirs(OUT, exist_ok=True)

    fetch_json_specs("gfm", ["commonmark.0.31.json", "gfm.0.29.json"], "gfm.json")

    original_pairs = [
        "amps_and_angles_encoding", "auto_links", "backslash_escapes",
        "blockquotes_with_code_blocks", "code_blocks", "code_spans",
        "hard_wrapped_paragraphs_with_list_like_lines", "horizontal_rules",
        "inline_html_advanced", "inline_html_comments", "inline_html_simple",
        "links_inline_style", "links_reference_style", "links_shortcut_references",
        "literal_quotes_in_titles", "nested_blockquotes",
        "ordered_and_unordered_lists", "tabs", "tidyness",
    ]
    fetch_md_html_specs("original", original_pairs, "original.json")

    api_url = "https://api.github.com/repos/markedjs/marked/contents/test/specs/new?ref=v13.0.0"
    with urllib.request.urlopen(api_url) as resp:
        entries = json.loads(resp.read().decode("utf-8"))
    stems = sorted({
        e["name"].rsplit(".", 1)[0]
        for e in entries
        if e["name"].endswith(".md")
    })
    tests = []
    for stem in stems:
        try:
            md = fetch(f"{BASE}/new/{stem}.md")
            html = fetch(f"{BASE}/new/{stem}.html")
            tests.append({"markdown": md, "html": html, "section": stem})
            print(f"Fetched new/{stem}")
        except Exception as exc:
            print(f"Skip new/{stem}: {exc}")
    out_path = os.path.join(OUT, "new.json")
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(tests, f, indent=2)
    print(f"Wrote {out_path}: {len(tests)} tests")


if __name__ == "__main__":
    main()
