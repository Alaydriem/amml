use amml_abi::AmmlCameraKind;

/// The kind of a vanilla camera from the name in its camera component.
pub struct CameraKindNames;

impl CameraKindNames {
    /// The kind of the camera named `name`; a name the table does not know is `OTHER`.
    pub fn kind_of(name: &str) -> AmmlCameraKind {
        match name {
            "minecraft:first_person" => AmmlCameraKind::FIRST_PERSON,
            "minecraft:third_person" => AmmlCameraKind::THIRD_PERSON,
            "minecraft:third_person_front" => AmmlCameraKind::THIRD_PERSON_FRONT,
            _ => AmmlCameraKind::OTHER,
        }
    }
}
