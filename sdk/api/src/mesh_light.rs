/// The light a world mesh is drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MeshLight {
    /// The mesh colour as given, whatever the light of the world.
    Full,
    /// The light of the world at the mesh. Not served in this version (`NotAvailable`).
    World,
}
