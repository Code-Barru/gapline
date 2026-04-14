use comfy_table::{ContentArrangement, Table};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use serde_json::{Value, to_string_pretty};
use std::fmt::Write as _;
use std::io::{self, Write};
use std::path::Path;

use super::{
    HtmlDest, announce_html_dest, html_escape, open_html_sink, open_writer, write_csv_table,
    xml_writer_to_io,
};
use crate::cli::OutputFormat;
use headway_core::crud::read::ReadResult;

/// Renders read results in the specified output format.
///
/// # Errors
///
/// Returns an error if writing to the output destination fails.
pub fn render_read_results(
    result: &ReadResult,
    format: OutputFormat,
    output_dest: Option<&Path>,
) -> io::Result<()> {
    match format {
        OutputFormat::Text => render_text(result, output_dest),
        OutputFormat::Json => render_json(result, output_dest),
        OutputFormat::Csv => render_csv(result, output_dest),
        OutputFormat::Xml => render_xml(result, output_dest),
        OutputFormat::Html => render_html(result, output_dest),
    }
}

fn render_text(result: &ReadResult, output_dest: Option<&Path>) -> io::Result<()> {
    let mut writer = open_writer(output_dest)?;
    let count = result.rows.len();

    if count == 0 {
        writeln!(writer, "0 records found")?;
        return writer.flush();
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(&result.headers);

    for row in &result.rows {
        let cells: Vec<&str> = row.iter().map(|c| c.as_deref().unwrap_or("")).collect();
        table.add_row(cells);
    }

    writeln!(writer, "{table}")?;
    writeln!(writer, "Found {count} records in {}", result.file_name)?;
    writer.flush()
}

fn render_json(result: &ReadResult, output_dest: Option<&Path>) -> io::Result<()> {
    let records: Vec<Value> = result
        .rows
        .iter()
        .map(|row| {
            let obj: serde_json::Map<String, Value> = result
                .headers
                .iter()
                .zip(row.iter())
                .map(|(header, cell)| {
                    let value = match cell {
                        Some(v) => Value::String(v.clone()),
                        None => Value::Null,
                    };
                    ((*header).to_owned(), value)
                })
                .collect();
            Value::Object(obj)
        })
        .collect();

    let mut writer = open_writer(output_dest)?;
    writeln!(writer, "{}", to_string_pretty(&records)?)?;
    writer.flush()
}

fn render_csv(result: &ReadResult, output_dest: Option<&Path>) -> io::Result<()> {
    let rows = result.rows.iter().map(|row| {
        row.iter()
            .map(|c| c.as_deref().unwrap_or(""))
            .collect::<Vec<&str>>()
    });
    write_csv_table(output_dest, result.headers.iter().copied(), rows)
}

fn render_xml(result: &ReadResult, output_dest: Option<&Path>) -> io::Result<()> {
    let writer = open_writer(output_dest)?;
    let mut xml_w = quick_xml::Writer::new_with_indent(writer, b' ', 2);

    xml_w
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(xml_writer_to_io)?;

    let mut root = BytesStart::new("records");
    root.push_attribute(("file", result.file_name));
    xml_w
        .write_event(Event::Start(root))
        .map_err(xml_writer_to_io)?;

    for row in &result.rows {
        xml_w
            .write_event(Event::Start(BytesStart::new("record")))
            .map_err(xml_writer_to_io)?;
        for (header, cell) in result.headers.iter().zip(row.iter()) {
            let mut field = BytesStart::new("field");
            field.push_attribute(("name", *header));
            match cell {
                Some(v) => {
                    xml_w
                        .write_event(Event::Start(field))
                        .map_err(xml_writer_to_io)?;
                    xml_w
                        .write_event(Event::Text(BytesText::new(v)))
                        .map_err(xml_writer_to_io)?;
                    xml_w
                        .write_event(Event::End(BytesEnd::new("field")))
                        .map_err(xml_writer_to_io)?;
                }
                None => {
                    xml_w
                        .write_event(Event::Empty(field))
                        .map_err(xml_writer_to_io)?;
                }
            }
        }
        xml_w
            .write_event(Event::End(BytesEnd::new("record")))
            .map_err(xml_writer_to_io)?;
    }

    xml_w
        .write_event(Event::End(BytesEnd::new("records")))
        .map_err(xml_writer_to_io)?;

    let mut inner = xml_w.into_inner();
    writeln!(inner)?;
    inner.flush()
}

const READ_HTML_STYLE: &str = r#"<style>
*, *::before, *::after { box-sizing: border-box; }
body { margin: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #f5f5f7; color: #222; }
header { background: #1a1a2e; color: #fff; padding: 1.25rem 2rem; }
header h1 { margin: 0; font-size: 1.25rem; }
header .meta { opacity: .75; font-size: .85rem; margin-top: .25rem; }
main { max-width: 1200px; margin: 0 auto; padding: 1.5rem 2rem; }
table { width: 100%; border-collapse: collapse; background: #fff; box-shadow: 0 1px 3px rgba(0,0,0,.06); border-radius: 8px; overflow: hidden; }
thead { background: #eceff1; }
th, td { padding: .65rem .9rem; text-align: left; font-size: .9rem; border-bottom: 1px solid #f0f0f0; }
tbody tr:last-child td { border-bottom: none; }
tbody tr:hover { background: #fafbfc; }
td.empty { color: #aaa; font-style: italic; }
.count { color: #666; font-size: .85rem; margin-top: .75rem; }
</style>"#;

fn render_html(result: &ReadResult, output_dest: Option<&Path>) -> io::Result<()> {
    let mut html = String::new();
    let file_esc = html_escape(result.file_name);
    let rows = result.rows.len();
    let version = env!("CARGO_PKG_VERSION");
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    let _ = writeln!(html, "<title>Headway — {file_esc}</title>");
    html.push_str(READ_HTML_STYLE);
    html.push_str("\n</head>\n<body>\n");
    let _ = writeln!(
        html,
        "<header><h1>Headway Read Results</h1><div class=\"meta\">File: <strong>{file_esc}</strong> · {rows} records · headway v{version}</div></header>",
    );
    html.push_str("<main>\n");

    if result.rows.is_empty() {
        html.push_str("<p class=\"count\">0 records found.</p>\n");
    } else {
        html.push_str("<table>\n<thead><tr>");
        for h in &result.headers {
            let h_esc = html_escape(h);
            let _ = write!(html, "<th>{h_esc}</th>");
        }
        html.push_str("</tr></thead>\n<tbody>\n");
        for row in &result.rows {
            html.push_str("<tr>");
            for cell in row {
                match cell {
                    Some(v) => {
                        let v_esc = html_escape(v);
                        let _ = write!(html, "<td>{v_esc}</td>");
                    }
                    None => html.push_str("<td class=\"empty\">—</td>"),
                }
            }
            html.push_str("</tr>\n");
        }
        html.push_str("</tbody>\n</table>\n");
        let _ = writeln!(
            html,
            "<p class=\"count\">Found {rows} records in {file_esc}.</p>",
        );
    }

    html.push_str("</main>\n</body>\n</html>\n");

    let (mut writer, dest): (_, HtmlDest) = open_html_sink(output_dest)?;
    writer.write_all(html.as_bytes())?;
    writer.flush()?;
    drop(writer);
    announce_html_dest(&dest);
    Ok(())
}
