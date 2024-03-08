use crate::fluent_generated as fluent;
use rustc_errors::{
    codes::*, AddToDiagnostic, Applicability, DelayDm, Diag, DiagCtxt, EmissionGuarantee,
    IntoDiagnostic, Level, MultiSpan, SubdiagMessageOp,
};
use rustc_hir::def_id::DefId;
use rustc_hir::Node;
use rustc_infer::traits::ObjectSafetyViolation;
use rustc_macros::Diagnostic;
use rustc_middle::ty::{self, ClosureKind, PolyTraitRef, Ty, TyCtxt};
use rustc_span::{Span, Symbol};

#[derive(Diagnostic)]
#[diag(trait_selection_dump_vtable_entries)]
pub struct DumpVTableEntries<'a> {
    #[primary_span]
    pub span: Span,
    pub trait_ref: PolyTraitRef<'a>,
    pub entries: String,
}

#[derive(Diagnostic)]
#[diag(trait_selection_unable_to_construct_constant_value)]
pub struct UnableToConstructConstantValue<'a> {
    #[primary_span]
    pub span: Span,
    pub unevaluated: ty::UnevaluatedConst<'a>,
}

#[derive(Diagnostic)]
#[diag(trait_selection_empty_on_clause_in_rustc_on_unimplemented, code = E0232)]
pub struct EmptyOnClauseInOnUnimplemented {
    #[primary_span]
    #[label]
    pub span: Span,
}

#[derive(Diagnostic)]
#[diag(trait_selection_invalid_on_clause_in_rustc_on_unimplemented, code = E0232)]
pub struct InvalidOnClauseInOnUnimplemented {
    #[primary_span]
    #[label]
    pub span: Span,
}

#[derive(Diagnostic)]
#[diag(trait_selection_no_value_in_rustc_on_unimplemented, code = E0232)]
#[note]
pub struct NoValueInOnUnimplemented {
    #[primary_span]
    #[label]
    pub span: Span,
}

pub struct NegativePositiveConflict<'tcx> {
    pub impl_span: Span,
    pub trait_desc: ty::TraitRef<'tcx>,
    pub self_ty: Option<Ty<'tcx>>,
    pub negative_impl_span: Result<Span, Symbol>,
    pub positive_impl_span: Result<Span, Symbol>,
}

impl<G: EmissionGuarantee> IntoDiagnostic<'_, G> for NegativePositiveConflict<'_> {
    #[track_caller]
    fn into_diagnostic(self, dcx: &DiagCtxt, level: Level) -> Diag<'_, G> {
        let mut diag = Diag::new(dcx, level, fluent::trait_selection_negative_positive_conflict);
        diag.arg("trait_desc", self.trait_desc.print_only_trait_path().to_string());
        diag.arg("self_desc", self.self_ty.map_or_else(|| "none".to_string(), |ty| ty.to_string()));
        diag.span(self.impl_span);
        diag.code(E0751);
        match self.negative_impl_span {
            Ok(span) => {
                diag.span_label(span, fluent::trait_selection_negative_implementation_here);
            }
            Err(cname) => {
                diag.note(fluent::trait_selection_negative_implementation_in_crate);
                diag.arg("negative_impl_cname", cname.to_string());
            }
        }
        match self.positive_impl_span {
            Ok(span) => {
                diag.span_label(span, fluent::trait_selection_positive_implementation_here);
            }
            Err(cname) => {
                diag.note(fluent::trait_selection_positive_implementation_in_crate);
                diag.arg("positive_impl_cname", cname.to_string());
            }
        }
        diag
    }
}

#[derive(Diagnostic)]
#[diag(trait_selection_inherent_projection_normalization_overflow)]
pub struct InherentProjectionNormalizationOverflow {
    #[primary_span]
    pub span: Span,
    pub ty: String,
}

pub enum AdjustSignatureBorrow {
    Borrow { to_borrow: Vec<(Span, String)> },
    RemoveBorrow { remove_borrow: Vec<(Span, String)> },
}

impl AddToDiagnostic for AdjustSignatureBorrow {
    fn add_to_diagnostic_with<G: EmissionGuarantee, F: SubdiagMessageOp<G>>(
        self,
        diag: &mut Diag<'_, G>,
        _f: F,
    ) {
        match self {
            AdjustSignatureBorrow::Borrow { to_borrow } => {
                diag.arg("len", to_borrow.len());
                diag.multipart_suggestion_verbose(
                    fluent::trait_selection_adjust_signature_borrow,
                    to_borrow,
                    Applicability::MaybeIncorrect,
                );
            }
            AdjustSignatureBorrow::RemoveBorrow { remove_borrow } => {
                diag.arg("len", remove_borrow.len());
                diag.multipart_suggestion_verbose(
                    fluent::trait_selection_adjust_signature_remove_borrow,
                    remove_borrow,
                    Applicability::MaybeIncorrect,
                );
            }
        }
    }
}

#[derive(Diagnostic)]
#[diag(trait_selection_closure_kind_mismatch, code = E0525)]
pub struct ClosureKindMismatch {
    #[primary_span]
    #[label]
    pub closure_span: Span,
    pub expected: ClosureKind,
    pub found: ClosureKind,
    #[label(trait_selection_closure_kind_requirement)]
    pub cause_span: Span,

    pub trait_prefix: &'static str,

    #[subdiagnostic]
    pub fn_once_label: Option<ClosureFnOnceLabel>,

    #[subdiagnostic]
    pub fn_mut_label: Option<ClosureFnMutLabel>,
}

#[derive(Subdiagnostic)]
#[label(trait_selection_closure_fn_once_label)]
pub struct ClosureFnOnceLabel {
    #[primary_span]
    pub span: Span,
    pub place: String,
}

#[derive(Subdiagnostic)]
#[label(trait_selection_closure_fn_mut_label)]
pub struct ClosureFnMutLabel {
    #[primary_span]
    pub span: Span,
    pub place: String,
}

#[derive(Diagnostic)]
#[diag(trait_selection_async_closure_not_fn)]
pub(crate) struct AsyncClosureNotFn {
    #[primary_span]
    pub span: Span,
    pub kind: &'static str,
}

pub(crate) struct ObjectUnsafety<'tcx, 'a> {
    pub tcx: TyCtxt<'tcx>,
    pub span: Span,
    pub trait_def_id: DefId,
    pub violation: &'a ObjectSafetyViolation,
}

impl<'tcx, 'a> IntoDiagnostic<'tcx, ()> for ObjectUnsafety<'tcx, 'a> {
    fn into_diagnostic(self, dcx: &'tcx DiagCtxt, level: Level) -> Diag<'tcx, ()> {
        let mut err = Diag::new(
            dcx,
            level,
            DelayDm(|| {
                format!(
                    "the trait `{}` cannot be made into an object",
                    self.tcx.def_path_str(self.trait_def_id)
                )
            }),
        );
        let node = self.tcx.hir().get_if_local(self.trait_def_id);
        let mut spans = MultiSpan::from_span(self.span);
        if let Some(Node::Item(item)) = node {
            spans.push_span_label(item.ident.span, "this trait cannot be made into an object...");
            spans.push_span_label(self.span, format!("...because {}", self.violation.error_msg()));
        } else {
            spans.push_span_label(
                self.span,
                format!(
                    "the trait cannot be made into an object because {}",
                    self.violation.error_msg()
                ),
            );
        };
        err.span_note(
            spans,
            "for a trait to be \"object safe\" it needs to allow building a vtable to allow the \
            call to be resolvable dynamically; for more information visit \
            <https://doc.rust-lang.org/reference/items/traits.html#object-safety>",
        );
        if node.is_some() {
            // Only provide the help if its a local trait, otherwise it's not
            self.violation.solution().add_to(&mut err);
        }
        err
    }
}
