//! Strict, bounded CSV-to-Markdown conversion.
//!
//! AnyDoc's CSV frontend intentionally recovers by skipping unreadable
//! records. This local adapter uses a small RFC-4180 state machine instead:
//! malformed quoting and ragged rows are hard errors, and every allocation is
//! bounded by a field, row, column, and output budget.

use std::cell::RefCell;

use super::{DocumentError, MAX_MARKDOWN_SIZE};

const MAX_ROWS: usize = 100_000;
const MAX_COLUMNS: usize = 4_096;
const MAX_FIELD_BYTES: usize = 1_048_576;
const MAX_SAMPLE_ROWS: usize = 32;
const DELIMITERS: [u8; 4] = *b",;\t|";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseError {
    Malformed,
    Ragged,
    TooManyRows,
    TooManyColumns,
    FieldTooLarge,
    OutputTooLarge,
    SampleLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    StartField,
    Unquoted,
    Quoted,
    AfterQuote,
}

pub(super) fn to_markdown(bytes: &[u8]) -> Result<String, DocumentError> {
    let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
    std::str::from_utf8(bytes).map_err(|_| DocumentError::Malformed)?;
    if bytes.is_empty() {
        return Err(DocumentError::Malformed);
    }

    let delimiter = sniff_delimiter(bytes);
    let mut rows = 0usize;
    let mut expected_columns = None;
    parse_records(
        bytes,
        delimiter,
        MAX_FIELD_BYTES,
        |_row, column, _field| {
            if column >= MAX_COLUMNS {
                return Err(ParseError::TooManyColumns);
            }
            Ok(())
        },
        |_row, columns| {
            if columns == 0 {
                return Err(ParseError::Malformed);
            }
            if let Some(expected) = expected_columns {
                if expected != columns {
                    return Err(ParseError::Ragged);
                }
            } else {
                expected_columns = Some(columns);
            }
            rows = rows.checked_add(1).ok_or(ParseError::TooManyRows)?;
            if rows > MAX_ROWS {
                return Err(ParseError::TooManyRows);
            }
            Ok(())
        },
    )
    .map_err(map_error)?;

    let columns = expected_columns.ok_or(DocumentError::Malformed)?;
    let output = RefCell::new(String::new());
    parse_records(
        bytes,
        delimiter,
        MAX_FIELD_BYTES,
        |_row, column, field| {
            if column >= columns {
                return Err(ParseError::Ragged);
            }
            if column == 0 {
                push_bounded(&mut output.borrow_mut(), "|")?;
            }
            push_bounded(&mut output.borrow_mut(), " ")?;
            push_cell(&mut output.borrow_mut(), field)?;
            push_bounded(&mut output.borrow_mut(), " |")?;
            Ok(())
        },
        |row, row_columns| {
            if row_columns != columns {
                return Err(ParseError::Ragged);
            }
            if row == 0 {
                push_bounded(&mut output.borrow_mut(), "\n|")?;
                for _ in 0..columns {
                    push_bounded(&mut output.borrow_mut(), " --- |")?;
                }
            }
            push_bounded(&mut output.borrow_mut(), "\n")
        },
    )
    .map_err(map_error)?;
    Ok(output.into_inner())
}

fn map_error(error: ParseError) -> DocumentError {
    match error {
        ParseError::Malformed | ParseError::Ragged => DocumentError::Malformed,
        ParseError::TooManyRows | ParseError::TooManyColumns | ParseError::FieldTooLarge => {
            DocumentError::ResourceLimit
        }
        ParseError::OutputTooLarge => DocumentError::OutputTooLarge,
        ParseError::SampleLimit => DocumentError::Malformed,
    }
}

fn sniff_delimiter(bytes: &[u8]) -> u8 {
    let mut best = (b',', 0u32);
    for delimiter in DELIMITERS {
        let mut counts = Vec::new();
        let result = parse_records(
            bytes,
            delimiter,
            MAX_FIELD_BYTES,
            |_row, _column, _field| Ok(()),
            |_row, columns| {
                counts.push(columns);
                if counts.len() >= MAX_SAMPLE_ROWS {
                    Err(ParseError::SampleLimit)
                } else {
                    Ok(())
                }
            },
        );
        if !matches!(result, Ok(()) | Err(ParseError::SampleLimit)) {
            continue;
        }

        let mut modal = 0usize;
        let mut frequency = 0usize;
        for &candidate in &counts {
            let candidate_frequency = counts.iter().filter(|&&value| value == candidate).count();
            if (candidate_frequency, candidate) > (frequency, modal) {
                modal = candidate;
                frequency = candidate_frequency;
            }
        }
        if modal < 2 {
            continue;
        }
        let score = frequency as u32 * 1_000 + modal.min(500) as u32;
        if score > best.1 {
            best = (delimiter, score);
        }
    }
    best.0
}

fn parse_records<F, G>(
    bytes: &[u8],
    delimiter: u8,
    field_limit: usize,
    mut on_field: F,
    mut on_record: G,
) -> Result<(), ParseError>
where
    F: FnMut(usize, usize, &[u8]) -> Result<(), ParseError>,
    G: FnMut(usize, usize) -> Result<(), ParseError>,
{
    let mut field = Vec::new();
    let mut state = State::StartField;
    let mut row = 0usize;
    let mut column = 0usize;
    let mut record_started = false;
    let mut index = 0usize;

    while index < bytes.len() {
        let byte = bytes[index];
        match state {
            State::StartField => match byte {
                b'"' => {
                    record_started = true;
                    state = State::Quoted;
                    index += 1;
                }
                byte if byte == delimiter => {
                    record_started = true;
                    emit_field(&mut field, row, &mut column, &mut on_field)?;
                    index += 1;
                }
                b'\n' | b'\r' => {
                    if record_started {
                        emit_field(&mut field, row, &mut column, &mut on_field)?;
                        finish_record(&mut row, &mut column, &mut record_started, &mut on_record)?;
                    }
                    index += 1;
                    if byte == b'\r' && bytes.get(index) == Some(&b'\n') {
                        index += 1;
                    }
                }
                0 => return Err(ParseError::Malformed),
                byte => {
                    record_started = true;
                    push_field_byte(&mut field, byte, field_limit)?;
                    state = State::Unquoted;
                    index += 1;
                }
            },
            State::Unquoted => match byte {
                byte if byte == delimiter => {
                    emit_field(&mut field, row, &mut column, &mut on_field)?;
                    state = State::StartField;
                    index += 1;
                }
                b'\n' | b'\r' => {
                    emit_field(&mut field, row, &mut column, &mut on_field)?;
                    finish_record(&mut row, &mut column, &mut record_started, &mut on_record)?;
                    state = State::StartField;
                    index += 1;
                    if byte == b'\r' && bytes.get(index) == Some(&b'\n') {
                        index += 1;
                    }
                }
                b'"' | 0 => return Err(ParseError::Malformed),
                byte => {
                    push_field_byte(&mut field, byte, field_limit)?;
                    index += 1;
                }
            },
            State::Quoted => match byte {
                b'"' => {
                    state = State::AfterQuote;
                    index += 1;
                }
                0 => return Err(ParseError::Malformed),
                byte => {
                    push_field_byte(&mut field, byte, field_limit)?;
                    index += 1;
                }
            },
            State::AfterQuote => match byte {
                b'"' => {
                    push_field_byte(&mut field, b'"', field_limit)?;
                    state = State::Quoted;
                    index += 1;
                }
                byte if byte == delimiter => {
                    emit_field(&mut field, row, &mut column, &mut on_field)?;
                    state = State::StartField;
                    index += 1;
                }
                b'\n' | b'\r' => {
                    emit_field(&mut field, row, &mut column, &mut on_field)?;
                    finish_record(&mut row, &mut column, &mut record_started, &mut on_record)?;
                    state = State::StartField;
                    index += 1;
                    if byte == b'\r' && bytes.get(index) == Some(&b'\n') {
                        index += 1;
                    }
                }
                _ => return Err(ParseError::Malformed),
            },
        }
    }

    match state {
        State::StartField if record_started => {
            emit_field(&mut field, row, &mut column, &mut on_field)?;
            finish_record(&mut row, &mut column, &mut record_started, &mut on_record)?;
        }
        State::Unquoted | State::AfterQuote => {
            emit_field(&mut field, row, &mut column, &mut on_field)?;
            finish_record(&mut row, &mut column, &mut record_started, &mut on_record)?;
        }
        State::Quoted => return Err(ParseError::Malformed),
        State::StartField => {}
    }

    Ok(())
}

fn emit_field<F>(
    field: &mut Vec<u8>,
    row: usize,
    column: &mut usize,
    on_field: &mut F,
) -> Result<(), ParseError>
where
    F: FnMut(usize, usize, &[u8]) -> Result<(), ParseError>,
{
    on_field(row, *column, field)?;
    *column = column.checked_add(1).ok_or(ParseError::TooManyColumns)?;
    field.clear();
    Ok(())
}

fn finish_record<G>(
    row: &mut usize,
    column: &mut usize,
    record_started: &mut bool,
    on_record: &mut G,
) -> Result<(), ParseError>
where
    G: FnMut(usize, usize) -> Result<(), ParseError>,
{
    if *column == 0 {
        return Err(ParseError::Malformed);
    }
    on_record(*row, *column)?;
    *row = row.checked_add(1).ok_or(ParseError::TooManyRows)?;
    *column = 0;
    *record_started = false;
    Ok(())
}

fn push_field_byte(field: &mut Vec<u8>, byte: u8, field_limit: usize) -> Result<(), ParseError> {
    if field.len() >= field_limit {
        return Err(ParseError::FieldTooLarge);
    }
    field.push(byte);
    Ok(())
}

fn push_bounded(output: &mut String, value: &str) -> Result<(), ParseError> {
    if output.len().saturating_add(value.len()) > MAX_MARKDOWN_SIZE {
        return Err(ParseError::OutputTooLarge);
    }
    output.push_str(value);
    Ok(())
}

fn push_cell(output: &mut String, field: &[u8]) -> Result<(), ParseError> {
    let field = std::str::from_utf8(field).map_err(|_| ParseError::Malformed)?;
    for character in field.chars() {
        let escaped = match character {
            '|' => r"\|",
            '\\' => r"\\",
            '\n' | '\r' => " ",
            '<' => "&lt;",
            '>' => "&gt;",
            '[' => r"\[",
            ']' => r"\]",
            '*' => r"\*",
            '_' => r"\_",
            character if character == char::from(96) => "&#96;",
            _ => {
                let mut buffer = [0u8; 4];
                let value = character.encode_utf8(&mut buffer);
                push_bounded(output, value)?;
                continue;
            }
        };
        push_bounded(output, escaped)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_quoted_fields_and_escapes_markdown_structure() {
        let markdown = to_markdown(
            b"Account,Amount,Note\n\"Operating | cash\",1200,\"[remote](https://example.invalid)\"\n",
        )
        .unwrap();
        assert!(markdown.contains(r"Operating \| cash"));
        assert!(markdown.contains(r"\[remote\](https://example.invalid)"));
        assert!(markdown.contains("| --- | --- | --- |"));
    }

    #[test]
    fn sniffs_semicolon_and_preserves_multiline_fields_as_one_cell() {
        let markdown =
            to_markdown(b"Date;Memo;Amount\n2026-01-01;\"line one\nline two\";12\n").unwrap();
        assert!(markdown.contains("line one line two"));
        assert!(markdown.contains("| --- | --- | --- |"));
    }

    #[test]
    fn rejects_unclosed_quotes_and_ragged_rows() {
        assert!(matches!(
            to_markdown(b"a,b\n\"unclosed,x\n"),
            Err(DocumentError::Malformed)
        ));
        assert!(matches!(
            to_markdown(b"a,b\n1,2,3\n"),
            Err(DocumentError::Malformed)
        ));
    }

    #[test]
    fn rejects_invalid_utf8_and_oversized_fields() {
        assert!(matches!(
            to_markdown(&[b'a', b',', 0xff]),
            Err(DocumentError::Malformed)
        ));
        let oversized = format!("a\n{}\n", "x".repeat(MAX_FIELD_BYTES + 1));
        assert!(matches!(
            to_markdown(oversized.as_bytes()),
            Err(DocumentError::ResourceLimit)
        ));
    }

    #[test]
    fn ignores_blank_records_but_rejects_an_empty_document() {
        assert!(to_markdown(b"\n\n").is_err());
        let markdown = to_markdown(b"a,b\n\n1,2\n").unwrap();
        assert!(markdown.contains("1"));
    }
}
