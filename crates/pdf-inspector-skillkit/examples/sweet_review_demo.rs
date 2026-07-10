//! Run the Sweet tax-review demo package without starting the MCP server.
//!
//! Usage:
//!   cargo run -p pdf-inspector-skillkit --example sweet_review_demo
//!   cargo run -p pdf-inspector-skillkit --example sweet_review_demo -- demo_1099_bundle

use pdf_inspector_skillkit::domain::sweet;

fn main() {
    let package_id = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "demo_1040_w2_schedule_c".to_string());

    println!("Available Sweet demo packages:");
    for package in sweet::list_tax_packages() {
        println!(
            "  - {} :: {} ({} checks, {} expected findings)",
            package.package_id, package.title, package.check_count, package.expected_findings
        );
    }

    println!("\nRunning review for: {package_id}\n");
    match sweet::render_review_memo(&package_id) {
        Ok(markdown) => println!("{markdown}"),
        Err(err) => {
            eprintln!("ERROR: {err}");
            std::process::exit(2);
        }
    }
}
