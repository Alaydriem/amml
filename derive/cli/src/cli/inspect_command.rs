use amml_data::{EntryId, RecipeLoader};

use super::{
    CallersArgs, DisasmArgs, DraftArgs, ExplainArgs, FieldsArgs, FunctionArgs, InspectArgs, InspectOp, PacketsArgs,
    ScanArgs, SimilarArgs, VtableArgs, XrefsArgs,
};
use crate::errors::CliError;
use crate::services::{
    CodeXref, EntryExplainer, FieldAccess, FunctionListing, FunctionMatcher, ImageLoader, PacketTable, PatternDrafter,
    PatternScanner, RttiIndex, SlotNamer, StringXref, VtableReader,
};
use crate::structs::{CodeRef, Image, MatchCandidate, Pattern, RecipeTrace};

const LISTED_STRINGS: usize = 40;

pub struct InspectCommand;

impl InspectCommand {
    /// Runs one read-only operation on the executable and returns its text.
    pub fn run(args: &InspectArgs) -> Result<String, CliError> {
        let exe = args.exe.as_deref().ok_or(CliError::MissingExe)?;
        let image = ImageLoader::from_path(exe)?;
        Self::execute(&image, &args.op)
    }

    /// Runs one operation on an image that is already loaded.
    pub fn execute(image: &Image, op: &InspectOp) -> Result<String, CliError> {
        match op {
            InspectOp::Scan(scan) => Self::scan(image, scan),
            InspectOp::Function(function) => Self::function(image, function),
            InspectOp::Disasm(disasm) => Self::disasm(image, disasm),
            InspectOp::Vtable(vtable) => Self::vtable(image, vtable),
            InspectOp::Xrefs(xrefs) => Self::xrefs(image, xrefs),
            InspectOp::Callers(callers) => Self::callers(image, callers),
            InspectOp::Draft(draft) => Self::draft(image, draft),
            InspectOp::Explain(explain) => Self::explain(image, explain),
            InspectOp::Similar(similar) => Self::similar(image, similar),
            InspectOp::Fields(fields) => Self::fields(image, fields),
            InspectOp::Packets(packets) => Self::packets(image, packets),
            InspectOp::BuildId(_) => Ok(image.build().file_stem()),
        }
    }

    fn scan(image: &Image, args: &ScanArgs) -> Result<String, CliError> {
        let pattern = Pattern::parse(&args.pattern)?;
        let hits = PatternScanner::find_all(image, &pattern);
        let mut lines = vec![format!("matches: {}", hits.len())];
        for rva in hits.iter().take(args.limit) {
            lines.push(match image.functions().containing(*rva) {
                Some((start, end)) => format!("{rva:#x}  in {start:#x}..{end:#x}"),
                None => format!("{rva:#x}  in no function"),
            });
        }
        Ok(lines.join("\n"))
    }

    fn function(image: &Image, args: &FunctionArgs) -> Result<String, CliError> {
        let rva = args.rva.rva(image)?;
        Ok(FunctionListing::new(image).ranges(rva)?)
    }

    fn disasm(image: &Image, args: &DisasmArgs) -> Result<String, CliError> {
        if args.count == Some(0) {
            return Err(CliError::ZeroCount);
        }
        let rva = args.rva.rva(image)?;
        Ok(FunctionListing::new(image).listing(rva, args.count)?)
    }

    fn vtable(image: &Image, args: &VtableArgs) -> Result<String, CliError> {
        let vtable = match (&args.rva, &args.class) {
            (Some(rva), _) => rva.rva(image)?,
            (None, Some(class)) => RttiIndex::new(image).vtable(class)?,
            (None, None) => return Err(CliError::NoVtableSource),
        };
        let xrefs = args.names.then(|| StringXref::new(image));
        let namer = xrefs.as_ref().map(|xrefs| SlotNamer::new(image, xrefs));
        let mut lines = vec![format!("vtable {vtable:#x}")];
        for (slot, target, is_function) in VtableReader::new(image).slots(vtable, args.from, args.to) {
            let name = namer.as_ref().map(|namer| match target.and_then(|rva| namer.name_of(rva)) {
                Some(name) => format!("  \"{name}\""),
                None => "  -".to_string(),
            });
            let target = target.map_or_else(|| "-".to_string(), |rva| format!("{rva:#x}"));
            let note = if is_function { "fn" } else { "not a function start" };
            lines.push(format!("{slot}  {target}  {note}{}", name.unwrap_or_default()));
        }
        Ok(lines.join("\n"))
    }

    fn xrefs(image: &Image, args: &XrefsArgs) -> Result<String, CliError> {
        match (&args.rva, &args.string) {
            (Some(rva), None) => {
                let refs = CodeXref::new(image).refs_to(rva.rva(image)?);
                Ok(Self::ref_lines(&refs))
            }
            (None, Some(text)) => {
                let starts = StringXref::new(image).functions_using(image, text)?;
                let mut lines = vec![format!("functions: {}", starts.len())];
                lines.extend(starts.iter().map(|start| format!("{start:#x}")));
                Ok(lines.join("\n"))
            }
            _ => Err(CliError::XrefsSource),
        }
    }

    fn callers(image: &Image, args: &CallersArgs) -> Result<String, CliError> {
        let refs = CodeXref::new(image).callers(args.rva.rva(image)?);
        Ok(Self::ref_lines(&refs))
    }

    fn draft(image: &Image, args: &DraftArgs) -> Result<String, CliError> {
        let rva = args.rva.rva(image)?;
        let (pattern, matches) = PatternDrafter::new(image).draft(rva, args.len)?;
        Ok(format!("{pattern}\nmatches: {matches}"))
    }

    fn explain(image: &Image, args: &ExplainArgs) -> Result<String, CliError> {
        let set = RecipeLoader::load_dir(&args.recipes)?;
        let traces = EntryExplainer::new(image, &set).explain(&EntryId(args.id.clone()))?;
        Ok(RecipeTrace::render(&traces))
    }

    fn fields(new: &Image, args: &FieldsArgs) -> Result<String, CliError> {
        let rows = FieldAccess::list(new, args.rva.rva(new)?)?;
        let old_rows = match (&args.old_exe, &args.old_rva) {
            (Some(path), Some(rva)) => {
                let old = ImageLoader::from_path(path)?;
                Some(FieldAccess::list(&old, rva.rva(&old)?)?)
            }
            _ => None,
        };
        Ok(FieldAccess::render(old_rows.as_deref(), &rows))
    }

    fn packets(new: &Image, args: &PacketsArgs) -> Result<String, CliError> {
        let rows = PacketTable::read(new, args.create_packet.rva(new)?)?;
        let old_rows = match (&args.old_exe, &args.old_create_packet) {
            (Some(path), Some(rva)) => {
                let old = ImageLoader::from_path(path)?;
                Some(PacketTable::read(&old, rva.rva(&old)?)?)
            }
            _ => None,
        };
        Ok(PacketTable::render(&rows, old_rows.as_ref()))
    }

    fn similar(new: &Image, args: &SimilarArgs) -> Result<String, CliError> {
        let old = ImageLoader::from_path(&args.old_exe)?;
        let rva = args.rva.rva(&old)?;
        let matcher = FunctionMatcher::new(&old, new);
        let (start, fingerprint) = matcher.old_fingerprint(rva)?;
        let candidates = matcher.candidates(rva, args.top)?;
        let mut lines = vec![format!(
            "old function {start:#x}: strings {}  imports {}  constants {}  calls {}  size {}",
            fingerprint.strings.len(),
            fingerprint.imports.len(),
            fingerprint.constants.len(),
            fingerprint.calls,
            fingerprint.size
        )];
        lines.extend(fingerprint.strings.iter().take(LISTED_STRINGS).map(|text| format!("  string {text:?}")));
        if fingerprint.strings.len() > LISTED_STRINGS {
            lines.push(format!("  ... {} more strings", fingerprint.strings.len() - LISTED_STRINGS));
        }
        lines.extend(fingerprint.imports.iter().map(|name| format!("  import {name}")));
        lines.push(format!("candidates: {}", candidates.len()));
        lines.extend(candidates.iter().map(MatchCandidate::line));
        Ok(lines.join("\n"))
    }

    fn ref_lines(refs: &[CodeRef]) -> String {
        let mut lines = vec![format!("refs: {}", refs.len())];
        lines.extend(refs.iter().map(CodeRef::line));
        lines.join("\n")
    }
}
