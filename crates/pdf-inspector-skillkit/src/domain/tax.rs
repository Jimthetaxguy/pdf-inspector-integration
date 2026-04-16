use regex::Regex;
use serde::Serialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxFormType {
    W2,
    Form1099Int,
    Form1099Div,
    Form1099Misc,
    Form1099Nec,
    K1_1065,
    K1_1120S,
    Form1040,
    Form1099Composite,
    ScheduleA,
    ScheduleC,
    ScheduleD,
    ScheduleE,
    Unknown,
}

#[derive(Debug, Serialize)]
pub struct TaxFormIdentification {
    pub form_type: TaxFormType,
    pub confidence: f32,
    pub raw_match: String,
}

// All regexes are compile-time invariants — patterns are static literals,
// so a failure to compile is a programmer bug, not a runtime condition.
// `OnceLock` defers compilation to first use and caches the result for
// the program's lifetime, eliminating per-call regex construction.

macro_rules! static_regex {
    ($name:ident, $pattern:literal) => {
        fn $name() -> &'static Regex {
            static RE: OnceLock<Regex> = OnceLock::new();
            RE.get_or_init(|| {
                Regex::new($pattern).expect(concat!(
                    stringify!($name),
                    " regex must compile (compile-time invariant)"
                ))
            })
        }
    };
}

static_regex!(form_1099_composite_re, r"(?i)Form\s+1099\s+Composite");
static_regex!(w2_transcript_re, r"(?im)^#+\s*W-2\s+Transcript");
static_regex!(int_transcript_re, r"(?im)^#+\s*1099-INT\s+Transcript");
static_regex!(div_transcript_re, r"(?im)^#+\s*1099-DIV\s+Transcript");
static_regex!(misc_transcript_re, r"(?im)^#+\s*1099-MISC\s+Transcript");
static_regex!(nec_transcript_re, r"(?im)^#+\s*1099-NEC\s+Transcript");
static_regex!(k1_1065_re, r"(?i)Schedule\s+K-1.*Form\s+1065");
static_regex!(k1_1120s_re, r"(?i)Schedule\s+K-1.*Form\s+1120-S");
static_regex!(form_w2_re, r"(?i)Form\s+W-2");
static_regex!(wage_tax_statement_re, r"(?i)Wage\s+and\s+Tax\s+Statement");
static_regex!(form_1099_int_re, r"(?i)Form\s+1099-INT");
static_regex!(interest_income_re, r"(?i)Interest\s+Income");
static_regex!(form_1099_div_re, r"(?i)Form\s+1099-DIV");
static_regex!(
    dividends_distributions_re,
    r"(?i)Dividends\s+and\s+Distributions"
);
static_regex!(form_1099_misc_re, r"(?i)Form\s+1099-MISC");
static_regex!(form_1099_nec_re, r"(?i)Form\s+1099-NEC");
static_regex!(nonemployee_comp_re, r"(?i)Nonemployee\s+Compensation");
static_regex!(form_1040_re, r"(?i)Form\s+1040");
static_regex!(
    individual_income_re,
    r"(?i)U\.S\.\s*Individual\s+Income\s+Tax\s+Return"
);
static_regex!(schedule_a_re, r"(?i)Schedule\s+A.*Itemized\s+Deductions");
static_regex!(schedule_c_re, r"(?i)Schedule\s+C.*Profit\s+or\s+Loss");
static_regex!(schedule_d_re, r"(?i)Schedule\s+D.*Capital\s+Gains");
static_regex!(schedule_e_re, r"(?i)Schedule\s+E.*Supplemental\s+Income");

/// One row of the form-detection ladder. Order matters: the first
/// matching rule wins, so transcript headings (high specificity) come
/// before bare form-name mentions (lower specificity).
type Rule = (fn() -> &'static Regex, TaxFormType, f32);

const RULES: &[Rule] = &[
    (form_1099_composite_re, TaxFormType::Form1099Composite, 1.0),
    (w2_transcript_re, TaxFormType::W2, 0.95),
    (int_transcript_re, TaxFormType::Form1099Int, 0.95),
    (div_transcript_re, TaxFormType::Form1099Div, 0.95),
    (misc_transcript_re, TaxFormType::Form1099Misc, 0.95),
    (nec_transcript_re, TaxFormType::Form1099Nec, 0.95),
    (k1_1065_re, TaxFormType::K1_1065, 1.0),
    (k1_1120s_re, TaxFormType::K1_1120S, 1.0),
    (form_w2_re, TaxFormType::W2, 1.0),
    (wage_tax_statement_re, TaxFormType::W2, 0.9),
    (form_1099_int_re, TaxFormType::Form1099Int, 1.0),
    (interest_income_re, TaxFormType::Form1099Int, 0.7),
    (form_1099_div_re, TaxFormType::Form1099Div, 1.0),
    (dividends_distributions_re, TaxFormType::Form1099Div, 0.7),
    (form_1099_misc_re, TaxFormType::Form1099Misc, 1.0),
    (form_1099_nec_re, TaxFormType::Form1099Nec, 1.0),
    (nonemployee_comp_re, TaxFormType::Form1099Nec, 0.8),
    (form_1040_re, TaxFormType::Form1040, 1.0),
    (individual_income_re, TaxFormType::Form1040, 1.0),
    (schedule_a_re, TaxFormType::ScheduleA, 1.0),
    (schedule_c_re, TaxFormType::ScheduleC, 1.0),
    (schedule_d_re, TaxFormType::ScheduleD, 1.0),
    (schedule_e_re, TaxFormType::ScheduleE, 1.0),
];

fn match_rules(search_text: &str) -> TaxFormIdentification {
    for (re_fn, form_type, confidence) in RULES {
        if let Some(m) = re_fn().find(search_text) {
            return TaxFormIdentification {
                form_type: *form_type,
                confidence: *confidence,
                raw_match: m.as_str().to_string(),
            };
        }
    }
    TaxFormIdentification {
        form_type: TaxFormType::Unknown,
        confidence: 0.0,
        raw_match: String::new(),
    }
}

pub fn identify_tax_form(
    path: impl AsRef<std::path::Path>,
) -> Result<TaxFormIdentification, crate::SkillkitError> {
    let info = crate::process(path)?;
    let text = info.markdown.unwrap_or_default();
    let search_text = &text[..text.len().min(5000)];
    Ok(match_rules(search_text))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_pattern_hit(text: &str) -> (TaxFormType, f32) {
        // Mirrors the production ladder so the 2026-04-15 validation
        // fixes (TurboTax transcript headings + Form 1099 Composite)
        // can be locked in without real PDFs.
        let search_text = &text[..text.len().min(5000)];
        let id = match_rules(search_text);
        (id.form_type, id.confidence)
    }

    #[test]
    fn turbotax_w2_transcript_heading_matches() {
        let md = "# W-2 Transcript\n\nDisclaimer: ...";
        assert_eq!(first_pattern_hit(md), (TaxFormType::W2, 0.95));
    }

    #[test]
    fn turbotax_1099_div_transcript_heading_matches() {
        let md = "# 1099-DIV Transcript\n\nDisclaimer: ...";
        assert_eq!(first_pattern_hit(md), (TaxFormType::Form1099Div, 0.95));
    }

    #[test]
    fn broker_form_1099_composite_matches() {
        let md = "Some preamble\n\n# FORM 1099 COMPOSITE & YEAR-END SUMMARY\n";
        assert_eq!(first_pattern_hit(md), (TaxFormType::Form1099Composite, 1.0));
    }

    #[test]
    fn unknown_when_no_pattern_matches() {
        let md = "PAYEE NAME 1 ANY STREET\n0.00\n0.00";
        assert_eq!(first_pattern_hit(md), (TaxFormType::Unknown, 0.0));
    }
}
