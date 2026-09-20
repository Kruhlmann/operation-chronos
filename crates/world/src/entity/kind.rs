#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnitKind {
    Tank,
}

impl UnitKind {
    pub fn label(self) -> &'static str {
        match self {
            UnitKind::Tank => "TANK",
        }
    }
}
