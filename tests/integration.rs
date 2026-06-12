use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
struct SpecTest {
    markdown: String,
    html: String,
    #[serde(rename = "section")]
    _section: Option<String>,
    #[serde(rename = "example")]
    _example: Option<u32>,
}

fn run_spec_file(path: &str, threshold: f64) {
    let data = fs::read_to_string(path).expect("Spec file missing ? run make setup");
    let tests: Vec<SpecTest> = serde_json::from_str(&data).expect("Failed to parse spec JSON");

    let mut passed = 0usize;
    let mut failed = 0usize;

    for t in &tests {
        let got = marked_rs::parse(&t.markdown);
        let expected_norm = normalize_html(&t.html);
        let got_norm = normalize_html(&got);
        if expected_norm == got_norm {
            passed += 1;
        } else {
            failed += 1;
        }
    }

    let total = passed + failed;
    let pass_rate = passed as f64 / total as f64 * 100.0;
    println!("{path}: {passed}/{total} ({pass_rate:.1}%)");

    assert!(
        pass_rate >= threshold,
        "Pass rate {:.1}% for {} is below {:.0}% threshold",
        pass_rate,
        path,
        threshold
    );
}

fn normalize_html(s: &str) -> String {
    s.trim().replace("\r\n", "\n").replace("\r", "\n")
}

#[test]
fn marked_original_spec() {
    run_spec_file("tests/original/specs/marked/original.json", 90.0);
}

#[test]
fn marked_new_spec() {
    run_spec_file("tests/original/specs/marked/new.json", 90.0);
}

#[test]
fn marked_gfm_spec() {
    run_spec_file("tests/original/specs/marked/gfm.json", 95.0);
}
