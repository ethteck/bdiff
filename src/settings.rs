#[derive(
    serde::Deserialize, serde::Serialize, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy,
)]
pub enum ByteGrouping {
    One,
    Two,
    Four,
    #[default]
    Eight,
    Sixteen,
    ThirtyTwo,
}

impl ToString for ByteGrouping {
    fn to_string(&self) -> String {
        match self {
            Self::One => "One",
            Self::Two => "Two",
            Self::Four => "Four",
            Self::Eight => "Eight",
            Self::Sixteen => "Sixteen",
            Self::ThirtyTwo => "Thirty two",
        }
        .to_string()
    }
}

impl From<ByteGrouping> for usize {
    fn from(value: ByteGrouping) -> Self {
        match value {
            ByteGrouping::One => 1,
            ByteGrouping::Two => 2,
            ByteGrouping::Four => 4,
            ByteGrouping::Eight => 8,
            ByteGrouping::Sixteen => 16,
            ByteGrouping::ThirtyTwo => 32,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Settings {
    pub byte_grouping: ByteGrouping,
}
