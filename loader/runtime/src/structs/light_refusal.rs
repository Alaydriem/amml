/// Why the light book refuses a light.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum LightRefusal {
    #[error("every light slot is held by another mod")]
    Full,
}
