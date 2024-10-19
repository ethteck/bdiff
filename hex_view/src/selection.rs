#[derive(Clone, Debug, Default, PartialEq)]
pub struct HexViewSelectionRange {
    pub first: usize,
    pub second: usize,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub enum HexViewSelectionState {
    #[default]
    None,
    Selecting,
    Selected,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub enum HexViewSelectionSide {
    #[default]
    Hex,
    Ascii,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HexViewSelection {
    pub range: HexViewSelectionRange,
    pub state: HexViewSelectionState,
    pub side: HexViewSelectionSide,
}

impl HexViewSelection {
    pub fn start(&self) -> usize {
        self.range.first.min(self.range.second)
    }

    pub fn end(&self) -> usize {
        self.range.second.max(self.range.first)
    }

    pub(crate) fn contains(&self, pos: usize) -> bool {
        self.state != HexViewSelectionState::None && pos >= self.start() && pos <= self.end()
    }

    pub fn begin(&mut self, pos: usize, side: HexViewSelectionSide) {
        self.range.first = pos;
        self.range.second = pos;
        self.state = HexViewSelectionState::Selecting;
        self.side = side;
    }

    pub fn update(&mut self, pos: usize) {
        self.range.second = pos;
    }

    pub fn finalize(&mut self, pos: usize) {
        self.range.second = pos;
        self.state = HexViewSelectionState::Selected;
    }

    pub fn clear(&mut self) {
        self.range.first = 0;
        self.range.second = 0;
        self.state = HexViewSelectionState::None;
        self.side = HexViewSelectionSide::default();
    }

    pub fn adjust_cur_pos(&mut self, delta: isize) {
        self.range.first = (self.range.first as isize + delta).max(0) as usize;
        self.range.second = (self.range.second as isize + delta).max(0) as usize;
    }
}
