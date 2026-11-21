use std::path::PathBuf;

use amml_abi::AmmlDirKind;
use amml_api::testing::FakeGame;

#[test]
fn a_folder_the_loader_gives_is_returned_and_one_it_lacks_is_none() {
    let fake = FakeGame::new();
    fake.set_dir(AmmlDirKind::CONFIG, "C:/x");
    let dirs = fake.ctx().dirs();

    assert_eq!(dirs.config_dir(), Some(PathBuf::from("C:/x")));
    assert_eq!(dirs.data_dir(), None);
}
