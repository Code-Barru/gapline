//! Declarative macro for the common foreign-key rule pattern.
//!
//! Covers rules that: build a `HashSet<&str>` of valid parent PKs, iterate the
//! child collection, and emit a standard orphan-reference error. Rules with
//! custom severity, precheck, multi-source PKs, or conditional logic stay
//! hand-written.

/// Generates a `pub struct $Name; impl ValidationRule for $Name { ... }`
/// implementing a foreign-key check.
///
/// Kinds:
/// - `child_fk: <ident> (required)` — FK field is a required newtype `Id: AsRef<str>`.
/// - `child_fk: <ident> (optional)` — FK field is `Option<Id>`.
/// - `parent_pk: <ident> (required)` — PK field is a required newtype.
/// - `parent_pk: <ident> (optional)` — PK field is `Option<Id>`.
macro_rules! impl_fk_rule {
    (
        $struct:ident,
        child_file: $cfile:literal,
        child: feed.$cf:ident as $cb:ident,
        child_fk: $cfk:ident ($ckind:ident),
        parent_file: $pfile:literal,
        parent: feed.$pf:ident,
        parent_pk: $ppk:ident ($pkind:ident),
        parent_entity: $pent:literal $(,)?
    ) => {
        pub struct $struct;

        impl $crate::validation::ValidationRule for $struct {
            fn rule_id(&self) -> &'static str {
                super::RULE_ID
            }

            fn section(&self) -> &'static str {
                super::SECTION
            }

            fn severity(&self) -> $crate::validation::Severity {
                $crate::validation::Severity::Error
            }

            fn validate(
                &self,
                feed: &$crate::models::GtfsFeed,
            ) -> ::std::vec::Vec<$crate::validation::ValidationError> {
                let valid_ids: ::std::collections::HashSet<&str> =
                    impl_fk_rule!(@collect_parent feed, $pf, $ppk, $pkind);

                impl_fk_rule!(
                    @iter_child feed, $cf, $cb, $cfk, $ckind,
                    valid_ids, $cfile, $pfile, $pent
                )
            }
        }
    };

    (@collect_parent $feed:ident, $pf:ident, $ppk:ident, required) => {
        $feed.$pf.iter().map(|p| p.$ppk.as_ref()).collect()
    };

    (@collect_parent $feed:ident, $pf:ident, $ppk:ident, optional) => {
        $feed
            .$pf
            .iter()
            .filter_map(|p| p.$ppk.as_ref().map(::std::convert::AsRef::as_ref))
            .collect()
    };

    (
        @iter_child $feed:ident, $cf:ident, $cb:ident, $cfk:ident, required,
        $valid:ident, $cfile:literal, $pfile:literal, $pent:literal
    ) => {
        $feed
            .$cf
            .iter()
            .enumerate()
            .filter(|(_, $cb)| !$valid.contains($cb.$cfk.as_ref()))
            .map(|(i, $cb)| {
                let line = i + 2;
                $crate::validation::ValidationError::new(
                    super::RULE_ID,
                    super::SECTION,
                    $crate::validation::Severity::Error,
                )
                .message(format!(
                    "{} '{}' in {} line {} references non-existent {} in {}",
                    stringify!($cfk),
                    $cb.$cfk,
                    $cfile,
                    line,
                    $pent,
                    $pfile,
                ))
                .file($cfile)
                .line(line)
                .field(stringify!($cfk))
                .value($cb.$cfk.as_ref())
            })
            .collect()
    };

    (
        @iter_child $feed:ident, $cf:ident, $cb:ident, $cfk:ident, optional,
        $valid:ident, $cfile:literal, $pfile:literal, $pent:literal
    ) => {
        $feed
            .$cf
            .iter()
            .enumerate()
            .filter_map(|(i, $cb)| {
                let id = $cb.$cfk.as_ref()?;
                if $valid.contains(id.as_ref()) {
                    return None;
                }
                let line = i + 2;
                Some(
                    $crate::validation::ValidationError::new(
                        super::RULE_ID,
                        super::SECTION,
                        $crate::validation::Severity::Error,
                    )
                    .message(format!(
                        "{} '{id}' in {} line {line} references non-existent {} in {}",
                        stringify!($cfk),
                        $cfile,
                        $pent,
                        $pfile,
                    ))
                    .file($cfile)
                    .line(line)
                    .field(stringify!($cfk))
                    .value(id.as_ref()),
                )
            })
            .collect()
    };
}
