use amml_data::{Check, EntryId};

use crate::errors::StepError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FailureReason {
    AllRecipesFailed(Vec<StepError>),
    // the distinct values the recipes reached, ascending
    Disagree(Vec<u64>),
    CheckFailed(Check),
    // the first referenced entry that failed
    DependencyFailed(EntryId),
    // every recipe that succeeded reads a vtable slot, and none confirms it by another route
    VtableRouteOnly,
    // a function with only `section` checks, or a vslot with only `range` checks, needs two agreeing recipes;
    // this many agreed
    WeakChecksOnly { agreeing: u32 },
}
