use clap::Subcommand;

use super::{
    BuildIdArgs, CallersArgs, DisasmArgs, DraftArgs, ExplainArgs, FieldsArgs, FunctionArgs, PacketsArgs, ScanArgs,
    SimilarArgs, VtableArgs, XrefsArgs,
};

#[derive(Subcommand, Clone, Debug)]
pub enum InspectOp {
    /// Count the matches of a pattern and list where they are
    Scan(ScanArgs),
    /// Print the ranges of the function that holds an address
    Function(FunctionArgs),
    /// Print the instructions of a function from an address
    Disasm(DisasmArgs),
    /// List the code pointers in a vtable
    Vtable(VtableArgs),
    /// List the code and data that refer to an address, or the functions that load a string
    Xrefs(XrefsArgs),
    /// List the direct calls and jumps to an address
    Callers(CallersArgs),
    /// Draft a pattern from the code at an address, with the build-specific bytes as wildcards
    Draft(DraftArgs),
    /// Run the recipes of an entry step by step and show where each one stops
    Explain(ExplainArgs),
    /// Find the functions of this build that look like a function of another build
    Similar(SimilarArgs),
    /// List the [this+disp] reads, writes and leas of a function, lined up with the same function of another build
    Fields(FieldsArgs),
    /// List every packet that createPacket makes, with its id, name, vtable, size and handler slot
    Packets(PacketsArgs),
    /// Print the BuildId file stem
    BuildId(BuildIdArgs),
}
