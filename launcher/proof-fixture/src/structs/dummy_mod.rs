/// One dummy mod release of the proof matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DummyMod {
    pub name: &'static str,
    pub version: &'static str,
    pub amml_version: Option<&'static str>,
}

impl DummyMod {
    pub const NAMESPACE: &'static str = "proof";

    /// The five releases, in the order the registry lists them.
    pub const MATRIX: [DummyMod; 5] = [
        DummyMod { name: "plain", version: "1.0.0", amml_version: None },
        DummyMod { name: "plain", version: "1.1.0", amml_version: None },
        DummyMod { name: "gated", version: "1.0.0", amml_version: None },
        DummyMod { name: "gated", version: "2.0.0", amml_version: Some("26.60.6") },
        DummyMod { name: "onlygated", version: "1.0.0", amml_version: Some("26.62.0") },
    ];

    pub fn repo(&self) -> String {
        format!("proof/amml-{}", self.name)
    }

    pub fn id(&self) -> String {
        format!("@{}/{}", Self::NAMESPACE, self.name)
    }

    pub fn asset(&self) -> String {
        format!("{}-{}-windows-x64.zip", self.name, self.version)
    }
}
