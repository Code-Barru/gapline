//! Structured exit codes for scripting.
//!
//! Scripts wrapping headway need to distinguish "validation found errors"
//! from "failed to open the feed" from "user aborted". Each constant below
//! corresponds to one of those categories. The README documents the contract
//! under the "Exit Codes" section — keep both in sync.

/// Operation completed normally. Also used when the user aborted an
/// interactive prompt (they chose not to proceed — not an error).
pub const SUCCESS: i32 = 0;

/// Generic command failure: invalid `--where` query, validation errors,
/// render failure, unknown field in `--set`, PK/FK violation, etc.
pub const COMMAND_FAILED: i32 = 1;

/// Configuration error: malformed `headway.toml`, unknown key, type
/// mismatch. Distinguished so that scripts can surface config issues to the
/// user separately from runtime failures.
pub const CONFIG_ERROR: i32 = 2;

/// Input/output error: feed file not found, cannot read the archive,
/// cannot write the output file, permission denied.
pub const INPUT_ERROR: i32 = 3;

/// The operation matched nothing and nothing was written. Distinct from
/// `SUCCESS` so that a script can tell "update ran and changed 0 records"
/// from "update ran and changed N records".
pub const NO_CHANGES: i32 = 4;
