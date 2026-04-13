use comfy_table::{ContentArrangement, Table};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use serde_json::{Value, to_string_pretty};
use std::io::{self, Write};
use std::path::Path;

use super::{csv_to_io, open_writer, xml_writer_to_io};
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
    let writer = open_writer(output_dest)?;
    let mut csv_w = csv::Writer::from_writer(writer);
    csv_w.write_record(&result.headers).map_err(csv_to_io)?;
    for row in &result.rows {
        let cells: Vec<&str> = row.iter().map(|c| c.as_deref().unwrap_or("")).collect();
        csv_w.write_record(&cells).map_err(csv_to_io)?;
    }
    csv_w.flush()?;
    Ok(())
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
