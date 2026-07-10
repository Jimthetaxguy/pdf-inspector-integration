use serde::{Deserialize, Serialize};

use super::tax::TaxFormType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonStatus {
    Pass,
    Flag,
}

#[derive(Debug, Clone, Serialize)]
pub struct DemoPackageSummary {
    pub package_id: String,
    pub title: String,
    pub return_type: String,
    pub tax_year: u16,
    pub form_count: usize,
    pub source_doc_count: usize,
    pub check_count: usize,
    pub expected_findings: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaxPackageForm {
    pub id: String,
    pub form_type: TaxFormType,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceDocument {
    pub id: String,
    pub form_type: TaxFormType,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaxReviewPackage {
    pub package_id: String,
    pub title: String,
    pub return_type: String,
    pub tax_year: u16,
    pub forms: Vec<TaxPackageForm>,
    pub source_docs: Vec<SourceDocument>,
    pub line_items: Vec<PackageLineItem>,
    pub advisory_findings: Vec<ReviewFinding>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PackageLineItem {
    pub id: String,
    pub label: String,
    pub category: String,
    pub return_reference: String,
    pub source_reference: String,
    pub return_amount: i64,
    pub source_amount: i64,
    pub tolerance: i64,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LineComparisonInput {
    pub label: String,
    pub return_reference: String,
    pub source_reference: String,
    pub return_amount: i64,
    pub source_amount: i64,
    #[serde(default)]
    pub tolerance: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LineComparison {
    pub label: String,
    pub return_reference: String,
    pub source_reference: String,
    pub return_amount: i64,
    pub source_amount: i64,
    pub difference: i64,
    pub absolute_difference: i64,
    pub tolerance: i64,
    pub status: ComparisonStatus,
    pub severity: ReviewSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewFinding {
    pub id: String,
    pub severity: ReviewSeverity,
    pub category: String,
    pub title: String,
    pub detail: String,
    pub return_reference: String,
    pub source_reference: Option<String>,
    pub recommended_action: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewSummary {
    pub total_findings: usize,
    pub critical: usize,
    pub warning: usize,
    pub info: usize,
    pub checks_passed: usize,
    pub checks_total: usize,
    pub confidence_score: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewMemo {
    pub package_id: String,
    pub title: String,
    pub return_type: String,
    pub tax_year: u16,
    pub summary: ReviewSummary,
    pub findings: Vec<ReviewFinding>,
    pub skipped_checks: Vec<String>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SweetReviewError {
    #[error("unknown Sweet demo package: {0}")]
    UnknownPackage(String),
}

pub fn list_tax_packages() -> Vec<DemoPackageSummary> {
    demo_packages()
        .into_iter()
        .map(|package| {
            let expected_findings = review_tax_package(&package.package_id)
                .map(|memo| memo.summary.total_findings)
                .unwrap_or_default();
            DemoPackageSummary {
                package_id: package.package_id,
                title: package.title,
                return_type: package.return_type,
                tax_year: package.tax_year,
                form_count: package.forms.len(),
                source_doc_count: package.source_docs.len(),
                check_count: package.line_items.len() + package.advisory_findings.len(),
                expected_findings,
            }
        })
        .collect()
}

pub fn review_tax_package(package_id: &str) -> Result<ReviewMemo, SweetReviewError> {
    let package = demo_package(package_id)
        .ok_or_else(|| SweetReviewError::UnknownPackage(package_id.to_string()))?;

    let mut findings = Vec::new();
    let mut checks_passed = 0usize;

    for item in &package.line_items {
        let comparison = compare_line_items(LineComparisonInput {
            label: item.label.clone(),
            return_reference: item.return_reference.clone(),
            source_reference: item.source_reference.clone(),
            return_amount: item.return_amount,
            source_amount: item.source_amount,
            tolerance: Some(item.tolerance),
        });
        if comparison.status == ComparisonStatus::Pass {
            checks_passed += 1;
        } else {
            findings.push(ReviewFinding {
                id: format!("F-{:03}", findings.len() + 1),
                severity: comparison.severity,
                category: item.category.clone(),
                title: format!("{} does not tie", item.label),
                detail: comparison.message,
                return_reference: item.return_reference.clone(),
                source_reference: Some(item.source_reference.clone()),
                recommended_action: item.recommended_action.clone(),
                confidence: 0.92,
            });
        }
    }

    for advisory in &package.advisory_findings {
        let mut finding = advisory.clone();
        finding.id = format!("F-{:03}", findings.len() + 1);
        findings.push(finding);
    }

    let checks_total = package.line_items.len() + package.advisory_findings.len();
    let critical = findings
        .iter()
        .filter(|f| f.severity == ReviewSeverity::Critical)
        .count();
    let warning = findings
        .iter()
        .filter(|f| f.severity == ReviewSeverity::Warning)
        .count();
    let info = findings
        .iter()
        .filter(|f| f.severity == ReviewSeverity::Info)
        .count();

    let confidence_score = if checks_total == 0 {
        0.0
    } else {
        ((checks_passed as f32 / checks_total as f32) * 0.25 + 0.72).min(0.97)
    };

    Ok(ReviewMemo {
        package_id: package.package_id,
        title: package.title,
        return_type: package.return_type,
        tax_year: package.tax_year,
        summary: ReviewSummary {
            total_findings: findings.len(),
            critical,
            warning,
            info,
            checks_passed,
            checks_total,
            confidence_score,
        },
        findings,
        skipped_checks: vec![
            "Source PDF parsing is represented by synthetic structured values in this demo."
                .to_string(),
            "Prior-year comparison is package-specific until Harris sample returns are available."
                .to_string(),
        ],
        next_steps: vec![
            "Validate the same checks against sanitized Harris exports.".to_string(),
            "Replace synthetic line items with parsed values from identify_tax_form + region extraction."
                .to_string(),
            "Capture reviewer dispositions for confirmed / false positive / follow-up.".to_string(),
        ],
    })
}

pub fn compare_line_items(input: LineComparisonInput) -> LineComparison {
    let tolerance = input.tolerance.unwrap_or(0).max(0);
    let difference = input.return_amount - input.source_amount;
    let absolute_difference = difference.abs();
    let status = if absolute_difference <= tolerance {
        ComparisonStatus::Pass
    } else {
        ComparisonStatus::Flag
    };
    let severity = if status == ComparisonStatus::Pass {
        ReviewSeverity::Info
    } else if absolute_difference >= 1_000 {
        ReviewSeverity::Critical
    } else {
        ReviewSeverity::Warning
    };
    let message = if status == ComparisonStatus::Pass {
        format!(
            "{} ties within tolerance: return {} vs source {}.",
            input.label, input.return_amount, input.source_amount
        )
    } else {
        format!(
            "{} differs by {}: return {} at {} vs source {} at {}.",
            input.label,
            difference,
            input.return_amount,
            input.return_reference,
            input.source_amount,
            input.source_reference
        )
    };

    LineComparison {
        label: input.label,
        return_reference: input.return_reference,
        source_reference: input.source_reference,
        return_amount: input.return_amount,
        source_amount: input.source_amount,
        difference,
        absolute_difference,
        tolerance,
        status,
        severity,
        message,
    }
}

pub fn render_review_memo(package_id: &str) -> Result<String, SweetReviewError> {
    let memo = review_tax_package(package_id)?;
    let mut out = String::new();
    out.push_str(&format!(
        "# Tax Review Memo - {} ({}, TY {})\n\n",
        memo.title, memo.return_type, memo.tax_year
    ));
    out.push_str("## Summary\n");
    out.push_str(&format!(
        "- Findings: {} ({} critical, {} warning, {} info)\n",
        memo.summary.total_findings, memo.summary.critical, memo.summary.warning, memo.summary.info
    ));
    out.push_str(&format!(
        "- Checks passed: {} of {}\n",
        memo.summary.checks_passed, memo.summary.checks_total
    ));
    out.push_str(&format!(
        "- Confidence score: {:.0}%\n\n",
        memo.summary.confidence_score * 100.0
    ));

    out.push_str("## Findings\n");
    if memo.findings.is_empty() {
        out.push_str("- No findings above tolerance.\n");
    } else {
        for finding in &memo.findings {
            out.push_str(&format!(
                "### {} - {:?}: {}\n",
                finding.id, finding.severity, finding.title
            ));
            out.push_str(&format!("{}\n\n", finding.detail));
            out.push_str(&format!("- Return: {}\n", finding.return_reference));
            if let Some(source) = &finding.source_reference {
                out.push_str(&format!("- Source: {}\n", source));
            }
            out.push_str(&format!("- Action: {}\n\n", finding.recommended_action));
        }
    }

    out.push_str("## Skipped Checks\n");
    for skipped in &memo.skipped_checks {
        out.push_str(&format!("- {skipped}\n"));
    }
    out.push_str("\n## Next Steps\n");
    for step in &memo.next_steps {
        out.push_str(&format!("- {step}\n"));
    }
    Ok(out)
}

fn demo_package(package_id: &str) -> Option<TaxReviewPackage> {
    demo_packages()
        .into_iter()
        .find(|package| package.package_id == package_id)
}

fn demo_packages() -> Vec<TaxReviewPackage> {
    vec![
        demo_1040_w2_schedule_c(),
        demo_1120_c_corp(),
        demo_1065_partnership(),
        demo_1120s_s_corp(),
        demo_k1_partner(),
        demo_1099_bundle(),
    ]
}

fn form(id: &str, form_type: TaxFormType, label: &str) -> TaxPackageForm {
    TaxPackageForm {
        id: id.to_string(),
        form_type,
        label: label.to_string(),
    }
}

fn source(id: &str, form_type: TaxFormType, label: &str) -> SourceDocument {
    SourceDocument {
        id: id.to_string(),
        form_type,
        label: label.to_string(),
    }
}

struct LineSeed<'a> {
    id: &'a str,
    label: &'a str,
    category: &'a str,
    return_reference: &'a str,
    source_reference: &'a str,
    return_amount: i64,
    source_amount: i64,
    recommended_action: &'a str,
}

fn line(seed: LineSeed<'_>) -> PackageLineItem {
    PackageLineItem {
        id: seed.id.to_string(),
        label: seed.label.to_string(),
        category: seed.category.to_string(),
        return_reference: seed.return_reference.to_string(),
        source_reference: seed.source_reference.to_string(),
        return_amount: seed.return_amount,
        source_amount: seed.source_amount,
        tolerance: 0,
        recommended_action: seed.recommended_action.to_string(),
    }
}

fn advisory(
    severity: ReviewSeverity,
    category: &str,
    title: &str,
    detail: &str,
    return_reference: &str,
    recommended_action: &str,
) -> ReviewFinding {
    ReviewFinding {
        id: String::new(),
        severity,
        category: category.to_string(),
        title: title.to_string(),
        detail: detail.to_string(),
        return_reference: return_reference.to_string(),
        source_reference: None,
        recommended_action: recommended_action.to_string(),
        confidence: 0.74,
    }
}

fn demo_1040_w2_schedule_c() -> TaxReviewPackage {
    TaxReviewPackage {
        package_id: "demo_1040_w2_schedule_c".to_string(),
        title: "1040 with W-2, Schedule C, and interest income".to_string(),
        return_type: "Form 1040".to_string(),
        tax_year: 2025,
        forms: vec![
            form("return-1040", TaxFormType::Form1040, "Form 1040"),
            form("schedule-c", TaxFormType::ScheduleC, "Schedule C"),
            form("schedule-b", TaxFormType::Form1040, "Schedule B"),
        ],
        source_docs: vec![
            source("w2-abc", TaxFormType::W2, "W-2 - ABC Dental"),
            source(
                "1099-int-bank",
                TaxFormType::Form1099Int,
                "1099-INT - First Bank",
            ),
        ],
        line_items: vec![
            line(LineSeed {
                id: "wages-line-1a",
                label: "W-2 wages",
                category: "Source Document Mismatch",
                return_reference: "Form 1040 line 1a",
                source_reference: "W-2 Box 1 - ABC Dental",
                return_amount: 84_732,
                source_amount: 87_432,
                recommended_action: "Verify W-2 Box 1 and correct Form 1040 line 1a before filing.",
            }),
            line(LineSeed {
                id: "interest-schedule-b",
                label: "Interest income",
                category: "Source Document Tie-Out",
                return_reference: "Schedule B line 2",
                source_reference: "1099-INT Box 1 - First Bank",
                return_amount: 412,
                source_amount: 412,
                recommended_action: "No action needed; retain source tie-out in review file.",
            }),
        ],
        advisory_findings: vec![advisory(
            ReviewSeverity::Warning,
            "Reasonableness",
            "Schedule C net profit dropped materially",
            "Schedule C net profit is down 72% from the prior-year demo baseline.",
            "Schedule C line 31",
            "Ask preparer to document the business change or verify missing gross receipts.",
        )],
    }
}

fn demo_1120_c_corp() -> TaxReviewPackage {
    TaxReviewPackage {
        package_id: "demo_1120_c_corp".to_string(),
        title: "C corporation return with depreciation tie-out".to_string(),
        return_type: "Form 1120".to_string(),
        tax_year: 2025,
        forms: vec![form("return-1120", TaxFormType::Form1120, "Form 1120")],
        source_docs: vec![source(
            "fixed-asset-rollforward",
            TaxFormType::Form1120,
            "Fixed asset rollforward",
        )],
        line_items: vec![line(LineSeed {
            id: "depreciation",
            label: "Depreciation expense",
            category: "Return To Workpaper Tie-Out",
            return_reference: "Form 1120 line 20",
            source_reference: "Fixed asset rollforward current depreciation",
            return_amount: 18_400,
            source_amount: 21_900,
            recommended_action: "Tie Form 1120 depreciation to Form 4562 and the asset rollforward.",
        })],
        advisory_findings: vec![advisory(
            ReviewSeverity::Info,
            "Completeness",
            "Officer compensation review point",
            "Officer compensation is present; reviewer should confirm W-2 support and reasonableness.",
            "Form 1120 line 12",
            "Confirm wages tie to payroll records and related-party documentation.",
        )],
    }
}

fn demo_1065_partnership() -> TaxReviewPackage {
    TaxReviewPackage {
        package_id: "demo_1065_partnership".to_string(),
        title: "Partnership return with K-1 allocation checks".to_string(),
        return_type: "Form 1065".to_string(),
        tax_year: 2025,
        forms: vec![
            form("return-1065", TaxFormType::Form1065, "Form 1065"),
            form("partner-k1-a", TaxFormType::K1_1065, "Partner A K-1"),
            form("partner-k1-b", TaxFormType::K1_1065, "Partner B K-1"),
        ],
        source_docs: vec![source(
            "allocations-workpaper",
            TaxFormType::Form1065,
            "Partner allocation workpaper",
        )],
        line_items: vec![line(LineSeed {
            id: "ordinary-income-k1-total",
            label: "Ordinary business income allocated to K-1s",
            category: "K-1 Allocation Tie-Out",
            return_reference: "Form 1065 Schedule K line 1",
            source_reference: "Sum of Partner K-1 Box 1 amounts",
            return_amount: 125_000,
            source_amount: 122_500,
            recommended_action: "Reconcile Schedule K ordinary income to partner K-1 Box 1 totals.",
        })],
        advisory_findings: vec![advisory(
            ReviewSeverity::Warning,
            "Basis",
            "Partner basis support not attached",
            "The package includes K-1 allocations but no partner basis rollforward in the demo manifest.",
            "Partner K-1 package",
            "Request basis schedules before approving loss allocations or distributions.",
        )],
    }
}

fn demo_1120s_s_corp() -> TaxReviewPackage {
    TaxReviewPackage {
        package_id: "demo_1120s_s_corp".to_string(),
        title: "S corporation return with shareholder K-1 checks".to_string(),
        return_type: "Form 1120-S".to_string(),
        tax_year: 2025,
        forms: vec![
            form("return-1120s", TaxFormType::Form1120S, "Form 1120-S"),
            form("shareholder-k1", TaxFormType::K1_1120S, "Shareholder K-1"),
        ],
        source_docs: vec![source(
            "shareholder-ledger",
            TaxFormType::Form1120S,
            "Shareholder ownership ledger",
        )],
        line_items: vec![line(LineSeed {
            id: "k1-ordinary-income",
            label: "Shareholder ordinary income allocation",
            category: "Shareholder Allocation Tie-Out",
            return_reference: "Form 1120-S Schedule K line 1",
            source_reference: "Sum of shareholder K-1 Box 1 amounts",
            return_amount: 94_200,
            source_amount: 94_200,
            recommended_action: "No action needed; retain allocation tie-out.",
        })],
        advisory_findings: vec![advisory(
            ReviewSeverity::Warning,
            "Reasonable Compensation",
            "Officer wage/distribution pattern needs review",
            "S corporation distributions exceed officer wages in the demo data.",
            "Form 1120-S Schedule K and W-2 support",
            "Ask reviewer to confirm reasonable-compensation documentation.",
        )],
    }
}

fn demo_k1_partner() -> TaxReviewPackage {
    TaxReviewPackage {
        package_id: "demo_k1_partner".to_string(),
        title: "Recipient-side K-1 review".to_string(),
        return_type: "Schedule K-1 recipient package".to_string(),
        tax_year: 2025,
        forms: vec![form("k1-1065", TaxFormType::K1_1065, "Schedule K-1")],
        source_docs: vec![source(
            "client-organizer",
            TaxFormType::Unknown,
            "Client organizer notes",
        )],
        line_items: vec![line(LineSeed {
            id: "k1-income",
            label: "K-1 ordinary income",
            category: "K-1 Recipient Tie-Out",
            return_reference: "Schedule E page 2",
            source_reference: "Schedule K-1 Box 1",
            return_amount: 36_700,
            source_amount: 36_700,
            recommended_action: "No action needed; retain K-1 tie-out.",
        })],
        advisory_findings: vec![advisory(
            ReviewSeverity::Warning,
            "Passive Activity",
            "Passive/nonpassive classification requires support",
            "K-1 income is carried to Schedule E, but the demo package lacks participation support.",
            "Schedule E page 2",
            "Confirm material participation or passive classification with the reviewer.",
        )],
    }
}

fn demo_1099_bundle() -> TaxReviewPackage {
    TaxReviewPackage {
        package_id: "demo_1099_bundle".to_string(),
        title: "1099 bundle tie-out".to_string(),
        return_type: "1040 source document bundle".to_string(),
        tax_year: 2025,
        forms: vec![
            form("schedule-b", TaxFormType::Form1040, "Schedule B"),
            form("schedule-c", TaxFormType::ScheduleC, "Schedule C"),
        ],
        source_docs: vec![
            source(
                "1099-int",
                TaxFormType::Form1099Int,
                "1099-INT - First Bank",
            ),
            source("1099-div", TaxFormType::Form1099Div, "1099-DIV - Brokerage"),
            source("1099-nec", TaxFormType::Form1099Nec, "1099-NEC - ClientCo"),
        ],
        line_items: vec![
            line(LineSeed {
                id: "interest-total",
                label: "Interest total",
                category: "Source Document Tie-Out",
                return_reference: "Schedule B line 2",
                source_reference: "Sum of 1099-INT Box 1",
                return_amount: 1_240,
                source_amount: 1_240,
                recommended_action: "No action needed; retain tie-out.",
            }),
            line(LineSeed {
                id: "nonemployee-comp",
                label: "Nonemployee compensation",
                category: "Source Document Mismatch",
                return_reference: "Schedule C gross receipts",
                source_reference: "1099-NEC Box 1",
                return_amount: 48_000,
                source_amount: 52_000,
                recommended_action:
                    "Confirm whether the missing $4,000 is included elsewhere or omitted.",
            }),
        ],
        advisory_findings: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_packages_includes_all_sweet_demo_ids() {
        let ids: Vec<String> = list_tax_packages()
            .into_iter()
            .map(|package| package.package_id)
            .collect();
        assert_eq!(
            ids,
            vec![
                "demo_1040_w2_schedule_c",
                "demo_1120_c_corp",
                "demo_1065_partnership",
                "demo_1120s_s_corp",
                "demo_k1_partner",
                "demo_1099_bundle",
            ]
        );
    }

    #[test]
    fn compare_line_items_passes_within_tolerance() {
        let comparison = compare_line_items(LineComparisonInput {
            label: "Interest income".to_string(),
            return_reference: "Schedule B line 2".to_string(),
            source_reference: "1099-INT Box 1".to_string(),
            return_amount: 412,
            source_amount: 412,
            tolerance: Some(0),
        });
        assert_eq!(comparison.status, ComparisonStatus::Pass);
        assert_eq!(comparison.absolute_difference, 0);
    }

    #[test]
    fn compare_line_items_flags_material_mismatch() {
        let comparison = compare_line_items(LineComparisonInput {
            label: "W-2 wages".to_string(),
            return_reference: "Form 1040 line 1a".to_string(),
            source_reference: "W-2 Box 1".to_string(),
            return_amount: 84_732,
            source_amount: 87_432,
            tolerance: Some(0),
        });
        assert_eq!(comparison.status, ComparisonStatus::Flag);
        assert_eq!(comparison.severity, ReviewSeverity::Critical);
        assert_eq!(comparison.difference, -2_700);
    }

    #[test]
    fn review_tax_package_counts_findings() {
        let memo = review_tax_package("demo_1040_w2_schedule_c").unwrap();
        assert_eq!(memo.summary.total_findings, 2);
        assert_eq!(memo.summary.critical, 1);
        assert_eq!(memo.summary.warning, 1);
        assert_eq!(memo.summary.checks_passed, 1);
    }

    #[test]
    fn render_review_memo_includes_reviewer_actions() {
        let memo = render_review_memo("demo_1099_bundle").unwrap();
        assert!(memo.contains("Tax Review Memo"));
        assert!(memo.contains("Nonemployee compensation"));
        assert!(memo.contains("Confirm whether the missing $4,000"));
    }

    #[test]
    fn unknown_package_errors() {
        let err = review_tax_package("missing").unwrap_err();
        assert!(err.to_string().contains("unknown Sweet demo package"));
    }
}
