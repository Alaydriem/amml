use std::path::PathBuf;
use std::time::{Duration, Instant};

use amml_data::{BuildDataStore, BuildIdReader, EntryId};
use amml_derive::cli::{
    CallersArgs, DeriveArgs, DeriveCommand, FunctionArgs, InspectArgs, InspectCommand, InspectOp, ScanArgs, VtableArgs,
    XrefsArgs,
};
use amml_derive::{Address, Image, ImageLoader, PacketTable};

// the 1.26.51.01 build; its executable is the same file before and after patching
const STEM: &str = "6aa482fd-2658118eba0dbac5";

struct Game;

impl Game {
    fn exe() -> PathBuf {
        match std::env::var("AMML_GAME_EXE") {
            Ok(path) => PathBuf::from(path),
            Err(error) => panic!("AMML_GAME_EXE: {error}"),
        }
    }

    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
    }

    fn image() -> Image {
        match ImageLoader::from_path(&Self::exe()) {
            Ok(image) => image,
            Err(error) => panic!("{error}"),
        }
    }
}

#[test]
#[ignore = "needs AMML_GAME_EXE"]
fn smoke_recipes_derive_on_the_real_exe_quickly_and_deterministically() {
    let work = tempfile::tempdir().unwrap();
    let args = DeriveArgs::new(
        Game::exe(),
        Game::root().join("fixtures").join("smoke-recipes"),
        work.path().join("data"),
        work.path().join("reports"),
        None,
        None,
        false,
    );
    let report = DeriveCommand::run(&args).unwrap();
    println!("{}", report.summary_line());
    assert_eq!(report.build.file_stem(), STEM);
    assert!(report.elapsed < Duration::from_secs(120));
    assert!(report.deterministic);
    assert_eq!(report.derived, report.total);

    let file = format!("{STEM}.hcl");
    let derived = BuildDataStore::read(&work.path().join("data").join(&file)).unwrap();
    let committed = BuildDataStore::read(&Game::root().join("data").join("smoke").join(&file)).unwrap();
    assert_eq!(derived, committed);
}

#[test]
#[ignore = "needs AMML_GAME_EXE"]
fn the_build_id_of_the_real_exe_is_pinned_and_shared_by_both_readers() {
    let from_file = BuildIdReader::from_path(&Game::exe()).unwrap();
    assert_eq!(from_file.file_stem(), STEM);
    assert_eq!(Game::image().build(), from_file);
}

#[test]
#[ignore = "needs AMML_GAME_EXE"]
fn chained_unwind_fragments_map_to_their_primary_and_do_so_on_every_load() {
    // three consecutive fragments chained to the same primary in the 1.26.51.01 `.pdata`
    const PRIMARY: u32 = 0x0E01_CD60;
    const FRAGMENTS: [(u32, u32); 3] =
        [(0x0E01_CD79, 0x0E01_CDE2), (0x0E01_CDE2, 0x0E01_CDFF), (0x0E01_CDFF, 0x0E01_CE10)];

    for image in [Game::image(), Game::image()] {
        let functions = image.functions();
        assert!(functions.is_start(PRIMARY));
        let ranges = functions.ranges_of(PRIMARY);
        assert_eq!(ranges[0].0, PRIMARY);
        for (start, end) in FRAGMENTS {
            assert!(!functions.is_start(start), "{start:#x} is a fragment, not a function start");
            assert_eq!(functions.containing(start), Some((PRIMARY, end)));
            assert_eq!(functions.containing(end - 1), Some((PRIMARY, end)));
            assert!(ranges.contains(&(start, end)), "{start:#x}..{end:#x} missing from {ranges:x?}");
        }
    }
}

#[test]
#[ignore = "needs AMML_GAME_EXE"]
fn get_mouse_grabbed_is_slot_147_from_all_three_recipes_including_slot_of() {
    let work = tempfile::tempdir().unwrap();
    let args = DeriveArgs::new(
        Game::exe(),
        Game::root().join("recipes"),
        work.path().join("data"),
        work.path().join("reports"),
        None,
        None,
        false,
    );
    let report = DeriveCommand::run(&args).unwrap();
    // every entry derives on 26.51: the layout differences are mirrors in amml-game, selected by entries that
    // resolve on every build
    let refused: Vec<&str> = report.failures.iter().map(|failure| failure.id.0.as_str()).collect();
    assert!(refused.is_empty(), "{refused:?}");
    assert_eq!(report.derived, report.total - refused.len());
    let derived = BuildDataStore::read(&work.path().join("data").join(format!("{STEM}.hcl"))).unwrap();
    let entry = derived.entries.get(&EntryId("ClientInstance.getMouseGrabbed".to_string())).unwrap();
    assert_eq!(entry.value, 147);
    assert_eq!(entry.recipes_agreeing, 3);
}

#[test]
#[ignore = "needs AMML_GAME_EXE"]
fn inspect_on_26_51() {
    // ClientInstance.update
    const UPDATE: u32 = 0x05da_7b30;
    // the first 16 bytes are a push sequence shared by thousands of functions, and the first 32 by 15
    const PREFIX_LEN: usize = 96;
    let run = |op: InspectOp| InspectCommand::run(&InspectArgs::new(Game::exe(), op)).unwrap();

    let function = run(InspectOp::Function(FunctionArgs { rva: Address::parse("0x5da7b30").unwrap() }));
    assert!(function.starts_with(&format!("primary {UPDATE:#x}..")), "{function}");

    let image = Game::image();
    let prefix = image.read(UPDATE, PREFIX_LEN).unwrap().iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>();
    let started = Instant::now();
    let scan = run(InspectOp::Scan(ScanArgs { pattern: prefix.join(" "), limit: 20 }));
    let elapsed = started.elapsed();
    println!("scan took {elapsed:?}");
    assert_eq!(scan.lines().next(), Some("matches: 1"), "{scan}");
    assert!(elapsed < Duration::from_secs(5));
}

#[test]
#[ignore = "needs AMML_GAME_EXE"]
fn xrefs_on_26_51() {
    // ClientInstance.update, which is only reached through its vtable slot
    const UPDATE: u32 = 0x05da_7b30;
    // the first slot of the ClientInstance vtable
    const VTABLE: u32 = 0x0e97_31b0;
    // a function that ClientInstance.update calls directly, and the call
    const CALLEE: u32 = 0x0028_9fe0;
    const CALL_SITE: u32 = 0x05da_7c10;
    let image = Game::image();
    let address = |rva: u32| Address::parse(&format!("{rva:#x}")).unwrap();
    let callers = |rva: u32| InspectCommand::execute(&image, &InspectOp::Callers(CallersArgs { rva: address(rva) }));
    let refs_to = |rva: u32| {
        InspectCommand::execute(&image, &InspectOp::Xrefs(XrefsArgs { rva: Some(address(rva)), string: None }))
    };

    let started = Instant::now();
    let found = callers(CALLEE).unwrap();
    let callers_time = started.elapsed();
    println!("callers took {callers_time:?}");
    let expected = format!("{CALL_SITE:#x}  call  in {UPDATE:#x}  call {CALLEE:#x}");
    assert!(found.lines().any(|line| line == expected), "{expected} not in the callers");
    assert!(callers_time < Duration::from_secs(30));

    let started = Instant::now();
    let refs = refs_to(CALLEE).unwrap();
    let refs_time = started.elapsed();
    println!("refs_to took {refs_time:?}");
    assert!(refs.lines().any(|line| line == expected));
    assert!(refs_time < Duration::from_secs(30));

    assert_eq!(callers(UPDATE).unwrap(), "refs: 0");
    let refs = refs_to(UPDATE).unwrap();
    // the vtable holds well under 600 slots
    let in_vtable = refs.lines().any(|line| {
        let mut words = line.split_whitespace();
        let site = words.next().and_then(|site| u32::from_str_radix(site.trim_start_matches("0x"), 16).ok());
        words.next() == Some("data-pointer") && site.is_some_and(|site| (VTABLE..VTABLE + 8 * 600).contains(&site))
    });
    assert!(in_vtable, "{refs}");
}

#[test]
#[ignore = "needs AMML_GAME_EXE"]
fn vtable_names_client_network_handler_2651() {
    // ClientNetworkHandler vtable on 1.26.51.01; slot 146 is handle(..., CameraPacket const&)
    let op = InspectOp::Vtable(VtableArgs {
        rva: Some(Address::parse("e7ae8d0").unwrap()),
        class: None,
        from: 146,
        to: 147,
        names: true,
    });
    let text = InspectCommand::execute(&Game::image(), &op).unwrap();
    println!("{text}");
    assert!(text.contains("ClientNetworkHandler::handle(const NetworkIdentifier &, const CameraPacket &)"), "{text}");
}

#[test]
#[ignore = "needs AMML_GAME_EXE"]
fn packets_2651_known_rows() {
    // MinecraftPackets.createPacket on 1.26.51.01
    let image = Game::image();
    let list = PacketTable::read(&image, 0x16ad850).unwrap();
    let row = |id: u32| list.rows.iter().find(|row| row.id == id).cloned().unwrap();
    let camera = row(300);
    assert_eq!(camera.name.as_deref(), Some("CameraInstructionPacket"));
    assert_eq!(
        (camera.vtable, camera.get_id, camera.size, camera.slot),
        (0xe945e00, 0x83bd1c0, Some(0x188), Some(147))
    );
    assert_eq!(row(328).slot, Some(33));
    assert_eq!(row(0xC6).slot, Some(145));
    assert_eq!(row(0x79).slot, Some(135));
    assert!(list.skipped * 10 < list.cases - list.empty, "skipped {}", list.skipped);
}
