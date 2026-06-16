use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
struct SpecTest {
    markdown: String,
    html: String,
    section: String,
    example: u32,
}

#[test]
fn commonmark_spec_compliance() {
    let data = fs::read_to_string("tests/original/specs/commonmark_spec.json")
        .expect("CommonMark spec file missing — run make setup");

    let tests: Vec<SpecTest> = serde_json::from_str(&data).expect("Failed to parse spec JSON");

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut failures: Vec<String> = Vec::new();

    let mut section_failures: std::collections::HashMap<String, u32> =
        std::collections::HashMap::new();

    for t in &tests {
        let got = marked_rs::parse(&t.markdown);
        let expected_norm = normalize_html(&t.html);
        let got_norm = normalize_html(&got);

        if expected_norm == got_norm {
            passed += 1;
        } else {
            failed += 1;
            *section_failures.entry(t.section.clone()).or_insert(0) += 1;
            if failures.len() < 50 {
                failures.push(format!(
                    "\n--- Example {} [{}] ---\
                     \nINPUT:    {:?}\
                     \nEXPECTED: {:?}\
                     \nGOT:      {:?}",
                    t.example, t.section, t.markdown, t.html, got
                ));
            }
        }
    }

    let total = passed + failed;
    let pass_rate = passed as f64 / total as f64 * 100.0;

    println!("\nCommonMark spec: {}/{} ({:.1}%)", passed, total, pass_rate);

    let mut by_section: std::collections::HashMap<String, (u32, u32)> = std::collections::HashMap::new();
    for t in &tests {
        let expected_norm = normalize_html(&t.html);
        let got = marked_rs::parse(&t.markdown);
        let got_norm = normalize_html(&got);
        
        let entry = by_section.entry(t.section.clone()).or_insert((0, 0));
        entry.1 += 1;
        if expected_norm == got_norm {
            entry.0 += 1;
        }
    }

    println!("\n## Section Compliance Table");
    println!("| Section | Passing | Total | Compliance |");
    println!("|---------|---------|-------|------------|");
    let mut sections: Vec<_> = by_section.into_iter().collect();
    sections.sort_by(|a, b| a.0.cmp(&b.0));
    for (sec, (pass, tot)) in sections {
        let pct = (pass as f64 / tot as f64) * 100.0;
        println!("| {} | {} | {} | {:.1}% |", sec, pass, tot, pct);
    }
    println!();

    if !section_failures.is_empty() {
        println!("\nFailures by section:");
        let mut sections: Vec<_> = section_failures.iter().collect();
        sections.sort_by(|a, b| b.1.cmp(a.1));
        for (section, count) in sections.iter().take(10) {
            println!("  {:40} {:3} failures", section, count);
        }
    }

    for f in &failures {
        eprintln!("{}", f);
    }

    assert!(
        pass_rate >= 95.0,
        "Pass rate {:.1}% is below 95% threshold.\nTop failing sections printed above.",
        pass_rate
    );
}

fn normalize_html(s: &str) -> String {
    s.trim().replace("\r\n", "\n").replace("\r", "\n")
}
