//! Validation runner for the 3 domain tools — tax / irc / sec.
//!
//! Usage:
//!   cargo run --example validate_domain -- tax <path>
//!   cargo run --example validate_domain -- irc <path>
//!   cargo run --example validate_domain -- sec <path>
//!
//! Outputs are intentionally redacted: only structural metadata, never
//! raw markdown or PII-bearing content.

use pdf_inspector_skillkit::domain;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: validate_domain <tax|irc|sec> <pdf_path>");
        std::process::exit(2);
    }
    let tool = args[1].as_str();
    let path = &args[2];

    println!("--- {tool} :: {path}", path = redact_home(path));

    match tool {
        "tax" => run_tax(path),
        "irc" => run_irc(path),
        "sec" => run_sec(path),
        other => {
            eprintln!("unknown tool: {other}");
            std::process::exit(2);
        }
    }
}

fn run_tax(path: &str) {
    match domain::tax::identify_tax_form(path) {
        Ok(id) => {
            println!("form_type: {:?}", id.form_type);
            println!("confidence: {}", id.confidence);
            println!("raw_match: {:?}", truncate(&id.raw_match, 80));
        }
        Err(e) => println!("ERROR: {e}"),
    }
}

fn run_irc(path: &str) {
    match domain::irc::parse_irc_sections(path) {
        Ok(r) => {
            println!("subtitle: {:?}", r.subtitle);
            println!("chapter: {:?}", r.chapter);
            println!("subchapter: {:?}", r.subchapter);
            println!("total_sections: {}", r.total_sections);
            let preview: Vec<String> = r
                .sections
                .iter()
                .take(8)
                .map(|s| format!("{} {}", s.section_number, truncate(&s.title, 50)))
                .collect();
            println!("first 8 section headings:");
            for line in preview {
                println!("  {line}");
            }
            let total_subs: usize = r.sections.iter().map(|s| s.subsections.len()).sum();
            println!("total subsections across all sections: {total_subs}");
        }
        Err(e) => println!("ERROR: {e}"),
    }
}

fn run_sec(path: &str) {
    match domain::sec::split_sec_filing(path) {
        Ok(sections) => {
            println!("total_sections: {}", sections.len());
            println!("first 12 sections:");
            for s in sections.iter().take(12) {
                println!(
                    "  [{}] {} ({} chars)",
                    s.item_number,
                    truncate(&s.name, 60),
                    s.char_length
                );
            }
        }
        Err(e) => println!("ERROR: {e}"),
    }
}

fn truncate(s: &str, max: usize) -> String {
    let cleaned: String = s.chars().filter(|c| !c.is_control()).collect();
    if cleaned.chars().count() <= max {
        cleaned
    } else {
        let head: String = cleaned.chars().take(max).collect();
        format!("{head}…")
    }
}

fn redact_home(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy().to_string();
        if let Some(rest) = path.strip_prefix(&home_str) {
            return format!("~{rest}");
        }
    }
    path.to_string()
}
