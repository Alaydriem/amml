use amml_runtime::{
    TextureBook, TextureJob, TextureKey, TextureRefusal, TextureRelease, TextureSourceKind, TextureTarget,
};

const MOD_DIR: &str = "C:\\mods\\peek";
const OWNER: usize = 1;
const OTHER: usize = 2;

fn pixels(width: u32, height: u32) -> Vec<u8> {
    vec![255; (width * height * 4) as usize]
}

// Runs the jobs of one pump as the game would: every bind and upload is answered with a held pointer.
fn answer(book: &TextureBook, now: u64) -> Vec<TextureJob> {
    let work = book.take_work(now);
    for job in &work.jobs {
        match job {
            TextureJob::Bind { key, .. } => assert!(book.bound(*key, 0x1000 + key.id as usize)),
            TextureJob::Upload { key, .. } => {
                book.uploaded(*key);
                assert!(book.bound(*key, 0x1000 + key.id as usize));
            }
        }
    }
    work.jobs
}

#[test]
fn ids_are_never_reused() {
    let book = TextureBook::new();
    let first = book.load(OWNER, TextureSourceKind::Vanilla, "textures/ui/a", MOD_DIR).unwrap();
    book.unload(OWNER, first).unwrap();

    let second = book.load(OWNER, TextureSourceKind::Vanilla, "textures/ui/a", MOD_DIR).unwrap();
    book.world_left();
    let third = book.load(OWNER, TextureSourceKind::Vanilla, "textures/ui/a", MOD_DIR).unwrap();

    assert!(first.id < second.id && second.id < third.id);
    assert_ne!(second.generation, third.generation);
}

#[test]
fn unload_releases_the_game_reference() {
    let book = TextureBook::new();
    let key = book.load(OWNER, TextureSourceKind::Vanilla, "textures/ui/a", MOD_DIR).unwrap();
    answer(&book, 0);

    book.unload(OWNER, key).unwrap();

    let work = book.take_work(1);
    assert_eq!(work.releases, vec![TextureRelease { held: 0x1000 + key.id as usize, unload_name: None }]);
    assert!(work.jobs.is_empty());
    assert_eq!(book.unload(OWNER, key), Err(TextureRefusal::Stale));
}

#[test]
fn an_uploaded_texture_is_unloaded_from_the_group_and_its_name_is_reused() {
    let book = TextureBook::new();
    let key = book.upload(OWNER, 2, 2, &pixels(2, 2)).unwrap();
    let jobs = answer(&book, 0);
    let TextureJob::Upload { name, .. } = &jobs[0] else { panic!("an upload job") };
    assert!(name.starts_with(TextureBook::NAME_PREFIX));

    book.unload(OWNER, key).unwrap();
    let next = book.upload(OWNER, 2, 2, &pixels(2, 2)).unwrap();

    let work = book.take_work(1);
    assert_eq!(work.releases[0].unload_name.as_deref(), Some(name.as_str()));
    let TextureJob::Upload { key: job_key, name: again, .. } = &work.jobs[0] else { panic!("an upload job") };
    assert_eq!((*job_key, again), (next, name));
}

#[test]
fn world_join_marks_every_texture_for_rebind() {
    let book = TextureBook::new();
    let file = book.load(OWNER, TextureSourceKind::Vanilla, "textures/ui/a", MOD_DIR).unwrap();
    let upload = book.upload(OWNER, 1, 1, &pixels(1, 1)).unwrap();
    answer(&book, 0);
    book.mark_ready(file);
    book.mark_ready(upload);
    assert!(book.ready(file) && book.ready(upload));

    book.rebind_all();

    assert!(!book.ready(file) && !book.ready(upload));
    let work = book.take_work(1);
    assert_eq!(work.releases.len(), 2);
    assert_eq!(work.jobs.len(), 2);
    assert!(matches!(work.jobs[0], TextureJob::Bind { .. }));
    assert!(matches!(work.jobs[1], TextureJob::Upload { .. }));
}

#[test]
fn not_ready_reasks_at_most_once_a_second() {
    let book = TextureBook::new();
    let key = book.load(OWNER, TextureSourceKind::Vanilla, "textures/ui/a", MOD_DIR).unwrap();
    assert_eq!(answer(&book, 0).len(), 1);

    assert!(book.take_work(500).jobs.is_empty());
    assert!(book.take_work(999).jobs.is_empty());
    let again = answer(&book, 1000);
    assert_eq!(again.len(), 1);
    assert!(book.take_work(1500).jobs.is_empty());
    assert_eq!(answer(&book, 2000).len(), 1);

    book.mark_ready(key);
    assert!(book.take_work(9000).jobs.is_empty());
}

#[test]
fn a_reask_that_gives_a_new_pointer_releases_the_old_one() {
    let book = TextureBook::new();
    let key = book.load(OWNER, TextureSourceKind::Vanilla, "textures/ui/a", MOD_DIR).unwrap();
    answer(&book, 0);

    let work = book.take_work(1000);
    assert!(book.bound(key, 0x9000));

    assert!(work.releases.is_empty());
    let released = book.take_work(1001).releases;
    assert_eq!(released, vec![TextureRelease { held: 0x1000 + key.id as usize, unload_name: None }]);
}

#[test]
fn disable_frees_only_that_mods_textures() {
    let book = TextureBook::new();
    let mine = book.load(OWNER, TextureSourceKind::Vanilla, "textures/ui/a", MOD_DIR).unwrap();
    let theirs = book.load(OTHER, TextureSourceKind::Vanilla, "textures/ui/b", MOD_DIR).unwrap();
    answer(&book, 0);

    assert_eq!(book.disable_owner(OWNER), 1);

    assert!(!book.is_current(mine));
    assert!(book.is_current(theirs));
    let work = book.take_work(1);
    assert_eq!(work.releases, vec![TextureRelease { held: 0x1000 + mine.id as usize, unload_name: None }]);
}

#[test]
fn another_mod_cannot_unload_a_texture() {
    let book = TextureBook::new();
    let key = book.load(OWNER, TextureSourceKind::Vanilla, "textures/ui/a", MOD_DIR).unwrap();

    assert_eq!(book.unload(OTHER, key), Err(TextureRefusal::NotOwner));
    assert!(book.is_current(key));
}

#[test]
fn mod_file_path_outside_the_mod_folder_is_refused() {
    let book = TextureBook::new();
    for path in ["../secret", "textures/../../secret", "/etc/passwd", "C:/Windows/x", "..\\x", "textures\\..\\..\\x"] {
        assert_eq!(
            book.load(OWNER, TextureSourceKind::ModFile, path, MOD_DIR),
            Err(TextureRefusal::OutsideModFolder),
            "{path}"
        );
    }
    assert_eq!(book.load(OWNER, TextureSourceKind::ModFile, "", MOD_DIR), Err(TextureRefusal::EmptyPath));
    assert_eq!(book.load(OWNER, TextureSourceKind::ModFile, "textures/a", ""), Err(TextureRefusal::OutsideModFolder));
    assert!(book.is_empty());
}

#[test]
fn a_mod_file_is_asked_for_by_its_absolute_path_without_the_extension() {
    let book = TextureBook::new();
    let key = book.load(OWNER, TextureSourceKind::ModFile, "textures/sort.png", MOD_DIR).unwrap();

    let work = book.take_work(0);

    assert_eq!(
        work.jobs,
        vec![TextureJob::Bind { key, target: TextureTarget::Raw("C:/mods/peek/textures/sort".to_string()) }]
    );
}

#[test]
fn pump_runs_at_most_four_uploads() {
    let book = TextureBook::new();
    for _ in 0..6 {
        book.upload(OWNER, 1, 1, &pixels(1, 1)).unwrap();
    }

    let first = answer(&book, 0);
    assert_eq!(first.len(), TextureBook::MAX_UPLOADS);
    assert!(book.has_due_work());

    let second = answer(&book, 1);
    assert_eq!(second.len(), 2);
}

#[test]
fn an_upload_of_the_wrong_size_is_refused() {
    let book = TextureBook::new();

    assert_eq!(book.upload(OWNER, 2, 2, &pixels(1, 1)), Err(TextureRefusal::BadImage));
    assert_eq!(book.upload(OWNER, 0, 2, &[]), Err(TextureRefusal::BadImage));
}

#[test]
fn world_left_makes_every_id_stale_and_gives_the_references_back() {
    let book = TextureBook::new();
    let key = book.load(OWNER, TextureSourceKind::Vanilla, "textures/ui/a", MOD_DIR).unwrap();
    answer(&book, 0);

    assert_eq!(book.world_left(), 1);

    assert!(!book.is_current(key));
    assert_eq!(book.drawable(key), Err(TextureRefusal::Stale));
    assert_eq!(book.unload(OWNER, key), Err(TextureRefusal::Stale));
    assert_eq!(book.take_work(1).releases.len(), 1);
    assert!(!book.bound(key, 0x5000));
}

#[test]
fn a_texture_is_drawable_only_once_ready() {
    let book = TextureBook::new();
    let key = book.load(OWNER, TextureSourceKind::Vanilla, "textures/ui/a", MOD_DIR).unwrap();
    assert_eq!(book.drawable(key), Ok(None));
    answer(&book, 0);
    assert_eq!(book.drawable(key), Ok(None));
    assert_eq!(book.waiting(), vec![(key, 0x1000 + key.id as usize)]);

    book.mark_ready(key);

    assert_eq!(book.drawable(key), Ok(Some(0x1000 + key.id as usize)));
    assert!(book.waiting().is_empty());
}

#[test]
fn a_foreign_generation_is_not_current() {
    let book = TextureBook::new();
    let key = book.load(OWNER, TextureSourceKind::Vanilla, "textures/ui/a", MOD_DIR).unwrap();

    assert!(!book.is_current(TextureKey { id: key.id, generation: key.generation + 1 }));
}

#[test]
fn a_texture_the_game_never_gives_waits_without_due_work() {
    let book = TextureBook::new();
    let key = book.load(OWNER, TextureSourceKind::Vanilla, "textures/ui/missing", MOD_DIR).unwrap();
    assert!(book.has_due_work() && !book.has_waiting());

    let work = book.take_work(0);
    assert!(book.bound(key, 0));

    assert_eq!(work.jobs.len(), 1);
    assert!(!book.has_due_work());
    assert!(book.has_waiting());
    assert!(book.take_work(500).jobs.is_empty());
}

#[test]
fn a_named_upload_keeps_its_name_in_the_job() {
    let book = TextureBook::new();

    let key = book.upload_named(OWNER, "font/glyph_E1", 2, 2, &pixels(2, 2)).unwrap();

    let work = book.take_work(0);
    assert!(matches!(
        &work.jobs[..],
        [TextureJob::Upload { key: job_key, name, width: 2, height: 2, .. }] if *job_key == key && name == "font/glyph_E1"
    ));
}

#[test]
fn a_named_upload_is_unloaded_by_name_and_the_name_is_not_handed_out_again() {
    let book = TextureBook::new();
    let key = book.upload_named(OWNER, "font/glyph_E1", 2, 2, &pixels(2, 2)).unwrap();
    answer(&book, 0);

    book.unload(OWNER, key).unwrap();
    let work = book.take_work(1);
    assert_eq!(work.releases.len(), 1);
    assert_eq!(work.releases[0].unload_name.as_deref(), Some("font/glyph_E1"));

    let plain = book.upload(OWNER, 2, 2, &pixels(2, 2)).unwrap();
    let jobs = book.take_work(2).jobs;
    assert!(
        matches!(&jobs[..], [TextureJob::Upload { key: job_key, name, .. }] if *job_key == plain && name.starts_with("amml/tex_"))
    );
}

#[test]
fn a_named_upload_needs_a_name_and_a_fitting_image() {
    let book = TextureBook::new();

    assert_eq!(book.upload_named(OWNER, " ", 2, 2, &pixels(2, 2)), Err(TextureRefusal::EmptyPath));
    assert_eq!(book.upload_named(OWNER, "font/glyph_E1", 2, 2, &[0; 3]), Err(TextureRefusal::BadImage));
}

#[test]
fn replacing_the_pixels_of_an_upload_uploads_again_in_place_with_no_unload() {
    let book = TextureBook::new();
    let key = book.upload_named(OWNER, "font/glyph_E1", 2, 2, &pixels(2, 2)).unwrap();
    answer(&book, 0);
    assert!(book.take_work(1).jobs.is_empty());

    book.replace_pixels(OWNER, key, 2, 2, &[7; 16]).unwrap();

    let work = book.take_work(2);
    assert!(work.releases.is_empty());
    assert!(matches!(
        &work.jobs[..],
        [TextureJob::Upload { key: job_key, name, pixels, .. }] if *job_key == key && name == "font/glyph_E1" && pixels == &[7; 16]
    ));
    // The answers of the pump give the old reference back and never the name.
    book.uploaded(key);
    assert!(book.bound(key, 0x2000));
    let work = book.take_work(3);
    assert!(work.releases.iter().all(|release| release.unload_name.is_none()));
    assert_eq!(work.releases.len(), 1);
}

#[test]
fn replacing_the_pixels_checks_the_key_the_owner_and_the_image() {
    let book = TextureBook::new();
    let key = book.upload(OWNER, 2, 2, &pixels(2, 2)).unwrap();
    let file = book.load(OWNER, TextureSourceKind::Vanilla, "textures/ui/a", "").unwrap();

    assert_eq!(book.replace_pixels(OTHER, key, 2, 2, &pixels(2, 2)), Err(TextureRefusal::NotOwner));
    assert_eq!(book.replace_pixels(OWNER, key, 2, 2, &[0; 3]), Err(TextureRefusal::BadImage));
    assert_eq!(book.replace_pixels(OWNER, file, 2, 2, &pixels(2, 2)), Err(TextureRefusal::BadImage));
    book.world_left();
    assert_eq!(book.replace_pixels(OWNER, key, 2, 2, &pixels(2, 2)), Err(TextureRefusal::Stale));
}

#[test]
fn with_drawable_runs_only_on_a_ready_held_texture() {
    let book = TextureBook::new();
    let key = book.upload(OWNER, 1, 1, &pixels(1, 1)).unwrap();
    let mut seen = Vec::new();

    assert_eq!(book.with_drawable(key, |held| seen.push(held)), Ok(None));
    answer(&book, 0);
    assert_eq!(book.with_drawable(key, |held| seen.push(held)), Ok(None));
    book.mark_ready(key);
    assert_eq!(book.with_drawable(key, |held| held + 1), Ok(Some(0x1000 + key.id as usize + 1)));
    book.world_left();

    assert_eq!(book.with_drawable(key, |held| seen.push(held)), Err(TextureRefusal::Stale));
    // `read` never ran for a texture that was not ready or was freed.
    assert!(seen.is_empty());
}
