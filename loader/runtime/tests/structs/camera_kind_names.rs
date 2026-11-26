use amml_abi::AmmlCameraKind;
use amml_runtime::CameraKindNames;

#[test]
fn vanilla_names_map_to_kinds() {
    assert_eq!(CameraKindNames::kind_of("minecraft:first_person"), AmmlCameraKind::FIRST_PERSON);
    assert_eq!(CameraKindNames::kind_of("minecraft:third_person"), AmmlCameraKind::THIRD_PERSON);
    assert_eq!(CameraKindNames::kind_of("minecraft:third_person_front"), AmmlCameraKind::THIRD_PERSON_FRONT);
}

#[test]
fn other_name_is_other() {
    for name in ["minecraft:free", "minecraft:follow_orbit", "first_person", "minecraft:first_person ", ""] {
        assert_eq!(CameraKindNames::kind_of(name), AmmlCameraKind::OTHER, "{name:?}");
    }
}
