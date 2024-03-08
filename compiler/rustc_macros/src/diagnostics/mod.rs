mod diagnostic;
mod diagnostic_builder;
mod error;
mod subdiagnostic;
mod utils;

use diagnostic::DiagnosticDerive;
use proc_macro2::TokenStream;
use subdiagnostic::SubdiagnosticDeriveBuilder;
use synstructure::Structure;

/// Implements `#[derive(Diagnostic)]`, which allows for errors to be specified as a struct,
/// independent from the actual diagnostics emitting code.
///
/// ```ignore (rust)
/// # extern crate rustc_errors;
/// # use rustc_errors::Applicability;
/// # extern crate rustc_span;
/// # use rustc_span::{symbol::Ident, Span};
/// # extern crate rust_middle;
/// # use rustc_middle::ty::Ty;
/// #[derive(Diagnostic)]
/// #[diag(borrowck_move_out_of_borrow, code = E0505)]
/// pub struct MoveOutOfBorrowError<'tcx> {
///     pub name: Ident,
///     pub ty: Ty<'tcx>,
///     #[primary_span]
///     #[label]
///     pub span: Span,
///     #[label(first_borrow_label)]
///     pub first_borrow_span: Span,
///     #[suggestion(code = "{name}.clone()")]
///     pub clone_sugg: Option<(Span, Applicability)>
/// }
/// ```
///
/// ```fluent
/// move_out_of_borrow = cannot move out of {$name} because it is borrowed
///     .label = cannot move out of borrow
///     .first_borrow_label = `{$ty}` first borrowed here
///     .suggestion = consider cloning here
/// ```
///
/// Then, later, to emit the error:
///
/// ```ignore (rust)
/// sess.emit_err(MoveOutOfBorrowError {
///     expected,
///     actual,
///     span,
///     first_borrow_span,
///     clone_sugg: Some(suggestion, Applicability::MachineApplicable),
/// });
/// ```
///
/// See rustc dev guide for more examples on using the `#[derive(Diagnostic)]`:
/// <https://rustc-dev-guide.rust-lang.org/diagnostics/diagnostic-structs.html>
pub fn session_diagnostic_derive(mut s: Structure<'_>) -> TokenStream {
    s.underscore_const(true);
    DiagnosticDerive::new(s).into_tokens()
}

/// Implements `#[derive(Subdiagnostic)]`, which allows for labels, notes, helps and
/// suggestions to be specified as a structs or enums, independent from the actual diagnostics
/// emitting code or diagnostic derives.
///
/// ```ignore (rust)
/// #[derive(Subdiagnostic)]
/// pub enum ExpectedIdentifierLabel<'tcx> {
///     #[label(expected_identifier)]
///     WithoutFound {
///         #[primary_span]
///         span: Span,
///     }
///     #[label(expected_identifier_found)]
///     WithFound {
///         #[primary_span]
///         span: Span,
///         found: String,
///     }
/// }
///
/// #[derive(Subdiagnostic)]
/// #[suggestion(style = "verbose",parser::raw_identifier)]
/// pub struct RawIdentifierSuggestion<'tcx> {
///     #[primary_span]
///     span: Span,
///     #[applicability]
///     applicability: Applicability,
///     ident: Ident,
/// }
/// ```
///
/// ```fluent
/// parser_expected_identifier = expected identifier
///
/// parser_expected_identifier_found = expected identifier, found {$found}
///
/// parser_raw_identifier = escape `{$ident}` to use it as an identifier
/// ```
///
/// Then, later, to add the subdiagnostic:
///
/// ```ignore (rust)
/// diag.subdiagnostic(ExpectedIdentifierLabel::WithoutFound { span });
///
/// diag.subdiagnostic(RawIdentifierSuggestion { span, applicability, ident });
/// ```
pub fn session_subdiagnostic_derive(mut s: Structure<'_>) -> TokenStream {
    s.underscore_const(true);
    SubdiagnosticDeriveBuilder::new().into_tokens(s)
}
