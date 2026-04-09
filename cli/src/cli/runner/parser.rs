//! `.hw` file parser — turns text lines into typed directives.

use std::path::{Path, PathBuf};

use clap::ValueEnum;

use super::error::RunError;
use crate::cli::parser::{CrudTarget, OutputFormat};

#[derive(Debug)]
pub struct HwDirective {
    pub line_number: usize,
    pub kind: DirectiveKind,
    pub raw_line: String,
}

#[derive(Debug)]
pub enum DirectiveKind {
    Feed {
        path: PathBuf,
    },
    Save {
        path: Option<PathBuf>,
    },
    Validate {
        format: Option<OutputFormat>,
        output: Option<PathBuf>,
    },
    Read {
        target: CrudTarget,
        where_query: Option<String>,
        format: Option<OutputFormat>,
        output: Option<PathBuf>,
    },
    Create {
        target: CrudTarget,
        set: Vec<String>,
        confirm: bool,
    },
    Update {
        target: CrudTarget,
        where_query: String,
        set: Vec<String>,
        confirm: bool,
        cascade: bool,
    },
    Delete {
        target: CrudTarget,
        where_query: Option<String>,
        confirm: bool,
    },
}

/// Parses a `.hw` file into a list of directives.
///
/// # Errors
///
/// Returns [`RunError`] on missing file or syntax errors.
pub fn parse_hw_file(path: &Path) -> Result<Vec<HwDirective>, RunError> {
    if !path.exists() {
        return Err(RunError::FileNotFound {
            path: path.display().to_string(),
        });
    }

    let content = std::fs::read_to_string(path)?;
    let mut directives = Vec::new();

    for (idx, raw_line) in content.lines().enumerate() {
        let line_number = idx + 1;
        let trimmed = raw_line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let tokens = tokenize(trimmed, line_number)?;
        let kind = parse_directive(&tokens, line_number)?;

        directives.push(HwDirective {
            line_number,
            kind,
            raw_line: trimmed.to_string(),
        });
    }

    Ok(directives)
}

fn tokenize(line: &str, line_number: usize) -> Result<Vec<String>, RunError> {
    let mut tokens = Vec::new();
    let mut chars = line.chars().peekable();
    let mut current = String::new();

    while let Some(&ch) = chars.peek() {
        match ch {
            '"' => {
                chars.next();
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some(c) => current.push(c),
                        None => {
                            return Err(RunError::Parse {
                                line: line_number,
                                message: "unterminated quoted string".to_string(),
                            });
                        }
                    }
                }
            }
            c if c.is_whitespace() => {
                chars.next();
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => {
                chars.next();
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    Ok(tokens)
}

fn parse_directive(tokens: &[String], line: usize) -> Result<DirectiveKind, RunError> {
    let command = tokens
        .first()
        .ok_or_else(|| RunError::Parse {
            line,
            message: "empty directive".to_string(),
        })?
        .as_str();

    match command {
        "feed" => parse_feed(tokens, line),
        "save" => Ok(parse_save(tokens, line)),
        "validate" => parse_validate(tokens, line),
        "read" => parse_read(tokens, line),
        "create" => parse_create(tokens, line),
        "update" => parse_update(tokens, line),
        "delete" => parse_delete(tokens, line),
        _ => Err(RunError::Parse {
            line,
            message: format!("unknown command '{command}'"),
        }),
    }
}

fn parse_feed(tokens: &[String], line: usize) -> Result<DirectiveKind, RunError> {
    if tokens.len() < 2 {
        return Err(RunError::Parse {
            line,
            message: "feed requires a path argument".to_string(),
        });
    }
    Ok(DirectiveKind::Feed {
        path: PathBuf::from(&tokens[1]),
    })
}

fn parse_save(tokens: &[String], _line: usize) -> DirectiveKind {
    let path = tokens.get(1).map(PathBuf::from);
    DirectiveKind::Save { path }
}

fn parse_validate(tokens: &[String], line: usize) -> Result<DirectiveKind, RunError> {
    let args = &tokens[1..];
    let mut format = None;
    let mut output = None;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                i += 1;
                format = Some(parse_output_format(args.get(i), line)?);
            }
            "-o" | "--output" => {
                i += 1;
                output = Some(PathBuf::from(require_arg(args, i, "-o", line)?));
            }
            other => {
                return Err(RunError::Parse {
                    line,
                    message: format!("unexpected argument '{other}' for validate"),
                });
            }
        }
        i += 1;
    }

    Ok(DirectiveKind::Validate { format, output })
}

fn parse_read(tokens: &[String], line: usize) -> Result<DirectiveKind, RunError> {
    if tokens.len() < 2 {
        return Err(RunError::Parse {
            line,
            message: "read requires a target (e.g. stops, routes)".to_string(),
        });
    }

    let target = parse_crud_target(&tokens[1], line)?;
    let args = &tokens[2..];
    let mut where_query = None;
    let mut format = None;
    let mut output = None;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-w" | "--where" => {
                i += 1;
                where_query = Some(require_arg(args, i, "--where", line)?.to_string());
            }
            "--format" => {
                i += 1;
                format = Some(parse_output_format(args.get(i), line)?);
            }
            "-o" | "--output" => {
                i += 1;
                output = Some(PathBuf::from(require_arg(args, i, "-o", line)?));
            }
            other => {
                return Err(RunError::Parse {
                    line,
                    message: format!("unexpected argument '{other}' for read"),
                });
            }
        }
        i += 1;
    }

    Ok(DirectiveKind::Read {
        target,
        where_query,
        format,
        output,
    })
}

fn parse_create(tokens: &[String], line: usize) -> Result<DirectiveKind, RunError> {
    if tokens.len() < 2 {
        return Err(RunError::Parse {
            line,
            message: "create requires a target (e.g. stops, routes)".to_string(),
        });
    }

    let target = parse_crud_target(&tokens[1], line)?;
    let args = &tokens[2..];
    let mut set = Vec::new();
    let mut confirm = false;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--set" => {
                i += 1;
                while i < args.len() && !args[i].starts_with('-') {
                    set.push(args[i].clone());
                    i += 1;
                }
                continue;
            }
            "--confirm" => confirm = true,
            other => {
                return Err(RunError::Parse {
                    line,
                    message: format!("unexpected argument '{other}' for create"),
                });
            }
        }
        i += 1;
    }

    if set.is_empty() {
        return Err(RunError::Parse {
            line,
            message: "create requires --set with field assignments".to_string(),
        });
    }

    Ok(DirectiveKind::Create {
        target,
        set,
        confirm,
    })
}

fn parse_update(tokens: &[String], line: usize) -> Result<DirectiveKind, RunError> {
    if tokens.len() < 2 {
        return Err(RunError::Parse {
            line,
            message: "update requires a target (e.g. stops, routes)".to_string(),
        });
    }

    let target = parse_crud_target(&tokens[1], line)?;
    let args = &tokens[2..];
    let mut where_query = None;
    let mut set = Vec::new();
    let mut confirm = false;
    let mut cascade = false;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-w" | "--where" => {
                i += 1;
                where_query = Some(require_arg(args, i, "--where", line)?.to_string());
            }
            "-s" | "--set" => {
                i += 1;
                while i < args.len() && !args[i].starts_with('-') {
                    set.push(args[i].clone());
                    i += 1;
                }
                continue;
            }
            "--confirm" => confirm = true,
            "--cascade" => cascade = true,
            other => {
                return Err(RunError::Parse {
                    line,
                    message: format!("unexpected argument '{other}' for update"),
                });
            }
        }
        i += 1;
    }

    let where_query = where_query.ok_or_else(|| RunError::Parse {
        line,
        message: "update requires --where filter".to_string(),
    })?;

    if set.is_empty() {
        return Err(RunError::Parse {
            line,
            message: "update requires --set with field assignments".to_string(),
        });
    }

    Ok(DirectiveKind::Update {
        target,
        where_query,
        set,
        confirm,
        cascade,
    })
}

fn parse_delete(tokens: &[String], line: usize) -> Result<DirectiveKind, RunError> {
    if tokens.len() < 2 {
        return Err(RunError::Parse {
            line,
            message: "delete requires a target (e.g. stops, routes)".to_string(),
        });
    }

    let target = parse_crud_target(&tokens[1], line)?;
    let args = &tokens[2..];
    let mut where_query = None;
    let mut confirm = false;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-w" | "--where" => {
                i += 1;
                where_query = Some(require_arg(args, i, "--where", line)?.to_string());
            }
            "--confirm" => confirm = true,
            other => {
                return Err(RunError::Parse {
                    line,
                    message: format!("unexpected argument '{other}' for delete"),
                });
            }
        }
        i += 1;
    }

    Ok(DirectiveKind::Delete {
        target,
        where_query,
        confirm,
    })
}

fn parse_crud_target(s: &str, line: usize) -> Result<CrudTarget, RunError> {
    CrudTarget::from_str(s, true).map_err(|_| RunError::Parse {
        line,
        message: format!("unknown target '{s}'"),
    })
}

fn parse_output_format(token: Option<&String>, line: usize) -> Result<OutputFormat, RunError> {
    let s = token.ok_or_else(|| RunError::Parse {
        line,
        message: "--format requires a value (json, csv, xml, text)".to_string(),
    })?;
    OutputFormat::from_str(s, true).map_err(|_| RunError::Parse {
        line,
        message: format!("unknown format '{s}'"),
    })
}

fn require_arg<'a>(
    args: &'a [String],
    i: usize,
    flag: &str,
    line: usize,
) -> Result<&'a str, RunError> {
    args.get(i)
        .map(String::as_str)
        .ok_or_else(|| RunError::Parse {
            line,
            message: format!("{flag} requires a value"),
        })
}
