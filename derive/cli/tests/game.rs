use std::path::PathBuf;
use std::time::{Duration, Instant};

use amml_data::{BuildData, BuildDataStore, BuildIdReader, EntryId};
use amml_derive::cli::{
    CallersArgs, FunctionArgs, InspectArgs, InspectCommand, InspectOp, ScanArgs, VtableArgs, XrefsArgs,
};
use amml_derive::{Address, Disasm, Image, ImageLoader, PacketTable};

// the unwind test looks at up to this many functions that have more than one range, spread over the image
const UNWIND_SAMPLE: usize = 50;

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

    // the committed data file of the build of the exe
    fn data() -> BuildData {
        let stem = match BuildIdReader::from_path(&Self::exe()) {
            Ok(build) => build.file_stem(),
            Err(error) => panic!("build id of {}: {error}", Self::exe().display()),
        };
        let path = Self::root().join("data").join(format!("{stem}.hcl"));
        if !path.is_file() {
            panic!("no data/{stem}.hcl for this exe; derive it first");
        }
        match BuildDataStore::read(&path) {
            Ok(data) => data,
            Err(error) => panic!("data/{stem}.hcl: {error}"),
        }
    }

    // the RVA a data entry holds
    fn anchor(name: &str) -> u32 {
        let data = Self::data();
        let Some(entry) = data.entries.get(&EntryId(name.to_string())) else {
            panic!("data/{}.hcl has no entry {name}", data.build.file_stem());
        };
        match u32::try_from(entry.value) {
            Ok(value) => value,
            Err(error) => panic!("{name} = {:#x}: {error}", entry.value),
        }
    }

    fn address(rva: u32) -> Address {
        match Address::parse(&format!("{rva:#x}")) {
            Ok(address) => address,
            Err(error) => panic!("{rva:#x}: {error}"),
        }
    }
}

#[test]
#[ignore = "needs AMML_GAME_EXE"]
fn the_build_id_of_the_real_exe_is_shared_by_both_readers_and_has_a_data_file() {
    let data = Game::data();
    let from_file = BuildIdReader::from_path(&Game::exe()).unwrap();
    assert_eq!(Game::image().build(), from_file);
    assert_eq!(data.build, from_file);
}

#[test]
#[ignore = "needs AMML_GAME_EXE"]
fn chained_unwind_fragments_map_to_their_primary_and_do_so_on_every_load() {
    let sample = |image: &Image| -> Vec<(u32, Vec<(u32, u32)>)> {
        let multi: Vec<_> = image.functions().grouped().into_iter().filter(|(_, ranges)| ranges.len() > 1).collect();
        let step = (multi.len() / UNWIND_SAMPLE).max(1);
        multi.into_iter().step_by(step).take(UNWIND_SAMPLE).collect()
    };
    let first = Game::image();
    let second = Game::image();
    let sampled = sample(&first);
    assert!(!sampled.is_empty(), "no function with more than one range");
    assert_eq!(sampled, sample(&second));

    for image in [&first, &second] {
        let functions = image.functions();
        for (primary, ranges) in &sampled {
            assert!(functions.is_start(*primary));
            assert_eq!(&functions.ranges_of(*primary), ranges);
            for &(start, end) in ranges.iter().filter(|range| range.0 != *primary) {
                assert!(!functions.is_start(start), "{start:#x} is a fragment, not a function start");
                assert_eq!(functions.containing(start), Some((*primary, end)));
                assert_eq!(functions.containing(end - 1), Some((*primary, end)));
            }
        }
    }
}

#[test]
#[ignore = "needs AMML_GAME_EXE"]
fn inspect_function_and_prefix_scan_on_client_instance_update() {
    // the first 16 bytes are a push sequence shared by thousands of functions, and the first 32 by 15
    const PREFIX_LEN: usize = 96;
    let update = Game::anchor("ClientInstance.update");
    let run = |op: InspectOp| InspectCommand::run(&InspectArgs::new(Game::exe(), op)).unwrap();

    let function = run(InspectOp::Function(FunctionArgs { rva: Game::address(update) }));
    assert!(function.starts_with(&format!("primary {update:#x}..")), "{function}");

    let image = Game::image();
    let prefix = image.read(update, PREFIX_LEN).unwrap().iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>();
    let started = Instant::now();
    let scan = run(InspectOp::Scan(ScanArgs { pattern: prefix.join(" "), limit: 20 }));
    let elapsed = started.elapsed();
    println!("scan took {elapsed:?}");
    assert_eq!(scan.lines().next(), Some("matches: 1"), "{scan}");
    assert!(elapsed < Duration::from_secs(5));
}

#[test]
#[ignore = "needs AMML_GAME_EXE"]
fn xrefs_of_client_instance_update_and_of_its_first_callee() {
    let update = Game::anchor("ClientInstance.update");
    let vtable = Game::anchor("ClientInstance.vtable");
    let image = Game::image();
    let callers =
        |rva: u32| InspectCommand::execute(&image, &InspectOp::Callers(CallersArgs { rva: Game::address(rva) }));
    let refs_to = |rva: u32| {
        InspectCommand::execute(&image, &InspectOp::Xrefs(XrefsArgs { rva: Some(Game::address(rva)), string: None }))
    };

    // the first direct call in ClientInstance.update
    let callee = Disasm::nth_call(&image, update, 0).unwrap();
    let started = Instant::now();
    let found = callers(callee).unwrap();
    let callers_time = started.elapsed();
    println!("callers took {callers_time:?}");
    let in_update = format!("in {update:#x}  call {callee:#x}");
    assert!(found.lines().any(|line| line.contains(&in_update)), "{in_update} not in the callers:\n{found}");
    assert!(callers_time < Duration::from_secs(30));

    let started = Instant::now();
    let refs = refs_to(callee).unwrap();
    let refs_time = started.elapsed();
    println!("refs_to took {refs_time:?}");
    assert!(
        refs.lines().any(|line| line.contains(&in_update)),
        "{in_update} not in the refs:
{refs}"
    );
    assert!(refs_time < Duration::from_secs(30));

    // ClientInstance.update is only reached through its vtable slot
    assert_eq!(callers(update).unwrap(), "refs: 0");
    let refs = refs_to(update).unwrap();
    // the vtable holds well under 600 slots
    let in_vtable = refs.lines().any(|line| {
        let mut words = line.split_whitespace();
        let site = words.next().and_then(|site| u32::from_str_radix(site.trim_start_matches("0x"), 16).ok());
        words.next() == Some("data-pointer") && site.is_some_and(|site| (vtable..vtable + 8 * 600).contains(&site))
    });
    assert!(in_vtable, "{refs}");
}

#[test]
#[ignore = "needs AMML_GAME_EXE"]
fn vtable_names_list_the_client_network_handler_camera_handler() {
    // the listing stops at the first slot that is not a code pointer, so 600 covers the whole vtable
    let op = InspectOp::Vtable(VtableArgs {
        rva: Some(Game::address(Game::anchor("ClientNetworkHandler.vtable"))),
        class: None,
        from: 0,
        to: 600,
        names: true,
    });
    let text = InspectCommand::execute(&Game::image(), &op).unwrap();
    assert!(text.contains("ClientNetworkHandler::handle(const NetworkIdentifier &, const CameraPacket &)"), "{text}");
}

#[test]
#[ignore = "needs AMML_GAME_EXE"]
fn packet_table_has_the_camera_instruction_row_with_every_column() {
    let create_packet = Game::anchor("MinecraftPackets.createPacket");
    let image = Game::image();
    let list = PacketTable::read(&image, create_packet).unwrap();
    let camera = list
        .rows
        .iter()
        .find(|row| row.name.as_deref() == Some("CameraInstructionPacket"))
        .unwrap_or_else(|| panic!("no CameraInstructionPacket row in {} rows", list.rows.len()));
    println!("{camera:?}");
    assert_ne!(camera.vtable, 0);
    assert_ne!(camera.get_id, 0);
    assert!(camera.size.is_some(), "{camera:?}");
    assert!(camera.slot.is_some(), "{camera:?}");
    assert!(list.skipped * 10 < list.cases - list.empty, "skipped {}", list.skipped);
}
