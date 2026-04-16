use regex::Regex;
use serde::Serialize;
use std::sync::OnceLock;

use crate::SkillkitError;

#[derive(Debug, Clone, Serialize)]
pub struct IrcSection {
    pub section_number: String,
    pub title: String,
    pub content: String,
    pub subsections: Vec<IrcSubsection>,
    pub char_offset: usize,
    pub char_length: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct IrcSubsection {
    pub label: String,
    pub title: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IrcParseResult {
    pub subtitle: Option<String>,
    pub chapter: Option<String>,
    pub subchapter: Option<String>,
    pub sections: Vec<IrcSection>,
    pub total_sections: usize,
}

// IRC structural patterns are static literals — compile once and reuse.
// All `expect` calls fire only on programmer error in the pattern itself,
// not on any runtime input.

fn section_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^(?:§\s*|SEC\.\s*|SECTION\s+)(\d+[A-Z]?)\.\s*(.*)")
            .expect("IRC section regex must compile (compile-time invariant)")
    })
}

fn subsection_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*\(([a-z])\)\s*(.*)")
            .expect("IRC subsection regex must compile (compile-time invariant)")
    })
}

fn subparagraph_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*\(([a-z])\)\s*\(([0-9]+)\)\s*(.*)")
            .expect("IRC subparagraph regex must compile (compile-time invariant)")
    })
}

fn subsubparagraph_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*\(([a-z])\)\s*\(([0-9]+)\)\s*\(([A-Z])\)\s*(.*)")
            .expect("IRC subsubparagraph regex must compile (compile-time invariant)")
    })
}

fn subtitle_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)SUBTITLE\s+([A-Z])")
            .expect("IRC subtitle regex must compile (compile-time invariant)")
    })
}

fn chapter_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)CHAPTER\s+([IVX0-9]+)")
            .expect("IRC chapter regex must compile (compile-time invariant)")
    })
}

fn subchapter_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)SUBCHAPTER\s+([A-Z])")
            .expect("IRC subchapter regex must compile (compile-time invariant)")
    })
}

pub fn parse_irc_sections(
    path: impl AsRef<std::path::Path>,
) -> Result<IrcParseResult, SkillkitError> {
    let info = crate::process(&path)?;
    let text = info.markdown.unwrap_or_default();

    if text.is_empty() {
        return Ok(IrcParseResult {
            subtitle: None,
            chapter: None,
            subchapter: None,
            sections: vec![],
            total_sections: 0,
        });
    }

    let path_str = path.as_ref().to_string_lossy().to_uppercase();

    let subtitle = extract_subtitle(&path_str, &text);
    let chapter = extract_chapter(&path_str, &text);
    let subchapter = extract_subchapter(&path_str, &text);

    let section_re = section_re();
    let subsection_re = subsection_re();
    let subparagraph_re = subparagraph_re();
    let subsubparagraph_re = subsubparagraph_re();

    let mut sections = Vec::new();
    let mut current_section: Option<(usize, String, String, String, Vec<IrcSubsection>)> = None;

    let line_endoffsets: Vec<usize> = text.match_indices('\n').map(|(i, _)| i).collect();

    fn get_line_offset(line_num: usize, line_endoffsets: &[usize]) -> usize {
        if line_num == 0 {
            0
        } else {
            line_endoffsets
                .get(line_num - 1)
                .map(|&o| o + 1)
                .unwrap_or(0)
        }
    }

    for (line_num, line) in text.lines().enumerate() {
        let line_offset = get_line_offset(line_num, &line_endoffsets);

        if let Some(caps) = section_re.captures(line) {
            if let Some((offset, sec_num, sec_title, sec_content, subs)) = current_section.take() {
                if !sec_content.trim().is_empty() {
                    sections.push(IrcSection {
                        section_number: format!("§{}", sec_num),
                        title: sec_title.trim().to_string(),
                        content: sec_content.trim().to_string(),
                        subsections: subs,
                        char_offset: offset,
                        char_length: sec_content.trim().len(),
                    });
                }
            }

            let sec_num = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let sec_title = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
            let mut sec_content = String::new();
            sec_content.push_str(line);
            sec_content.push('\n');

            current_section = Some((
                line_offset,
                sec_num.to_string(),
                sec_title,
                sec_content,
                Vec::new(),
            ));
        } else if let Some((
            _offset,
            ref _sec_num,
            ref _sec_title,
            ref mut sec_content,
            ref mut subs,
        )) = current_section
        {
            if let Some(caps) = subsubparagraph_re.captures(line) {
                let label = format!(
                    "({})({})({})",
                    caps.get(1).map(|m| m.as_str()).unwrap_or(""),
                    caps.get(2).map(|m| m.as_str()).unwrap_or(""),
                    caps.get(3).map(|m| m.as_str()).unwrap_or("")
                );
                let content = caps.get(4).map(|m| m.as_str()).unwrap_or("").to_string();
                subs.push(IrcSubsection {
                    label,
                    title: None,
                    content,
                });
                sec_content.push_str(line);
                sec_content.push('\n');
            } else if let Some(caps) = subparagraph_re.captures(line) {
                let label = format!(
                    "({})({})",
                    caps.get(1).map(|m| m.as_str()).unwrap_or(""),
                    caps.get(2).map(|m| m.as_str()).unwrap_or("")
                );
                let content = caps.get(3).map(|m| m.as_str()).unwrap_or("").to_string();
                subs.push(IrcSubsection {
                    label,
                    title: None,
                    content,
                });
                sec_content.push_str(line);
                sec_content.push('\n');
            } else if let Some(caps) = subsection_re.captures(line) {
                let label = format!("({})", caps.get(1).map(|m| m.as_str()).unwrap_or(""));
                let content = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
                subs.push(IrcSubsection {
                    label,
                    title: None,
                    content,
                });
                sec_content.push_str(line);
                sec_content.push('\n');
            } else {
                sec_content.push_str(line);
                sec_content.push('\n');
            }
        }
    }

    if let Some((offset, sec_num, sec_title, sec_content, subs)) = current_section {
        if !sec_content.trim().is_empty() {
            sections.push(IrcSection {
                section_number: format!("§{}", sec_num),
                title: sec_title.trim().to_string(),
                content: sec_content.trim().to_string(),
                subsections: subs,
                char_offset: offset,
                char_length: sec_content.trim().len(),
            });
        }
    }

    let total_sections = sections.len();

    Ok(IrcParseResult {
        subtitle,
        chapter,
        subchapter,
        sections,
        total_sections,
    })
}

fn extract_subtitle(path: &str, text: &str) -> Option<String> {
    let re = subtitle_re();
    if let Some(caps) = re.captures(path) {
        return Some(format!("Subtitle {}", caps.get(1)?.as_str()));
    }
    let first_lines: String = text.lines().take(5).collect::<Vec<_>>().join(" ");
    if let Some(caps) = re.captures(&first_lines) {
        return Some(format!("Subtitle {}", caps.get(1)?.as_str()));
    }
    None
}

fn extract_chapter(path: &str, text: &str) -> Option<String> {
    let re = chapter_re();
    if let Some(caps) = re.captures(path) {
        return Some(format!("Chapter {}", caps.get(1)?.as_str()));
    }
    let first_lines: String = text.lines().take(10).collect::<Vec<_>>().join(" ");
    if let Some(caps) = re.captures(&first_lines) {
        return Some(format!("Chapter {}", caps.get(1)?.as_str()));
    }
    None
}

fn extract_subchapter(path: &str, text: &str) -> Option<String> {
    let re = subchapter_re();
    if let Some(caps) = re.captures(path) {
        return Some(format!("Subchapter {}", caps.get(1)?.as_str()));
    }
    let first_lines: String = text.lines().take(15).collect::<Vec<_>>().join(" ");
    if let Some(caps) = re.captures(&first_lines) {
        return Some(format!("Subchapter {}", caps.get(1)?.as_str()));
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_nonexistent_file_returns_error() {
        let result = super::parse_irc_sections("/nonexistent/file.pdf");
        assert!(result.is_err());
    }
}
