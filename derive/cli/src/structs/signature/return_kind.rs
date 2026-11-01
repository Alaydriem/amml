#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnKind {
    // eax, ax or al is written on every path to a `ret`
    Int,
    // xmm0 is written on every path to a `ret`
    Float,
    None,
}

impl ReturnKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Float => "float",
            Self::None => "none",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        [Self::Int, Self::Float, Self::None].into_iter().find(|kind| kind.name() == name)
    }
}
