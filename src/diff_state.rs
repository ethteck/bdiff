use crate::hex_view::HexView;

#[derive(Debug)]
pub struct DiffState {
    pub enabled: bool,
    pub out_of_date: bool,
    pub diffs: Vec<bool>,
}

impl Default for DiffState {
    fn default() -> Self {
        Self {
            enabled: true,
            out_of_date: false,
            diffs: Vec::new(),
        }
    }
}

impl DiffState {
    pub fn is_diff_at(&self, index: usize) -> bool {
        if !self.enabled {
            return false;
        }

        if index >= self.diffs.len() {
            return false;
        }

        self.diffs[index]
    }

    pub fn get_next_diff(&self, start: usize) -> Option<usize> {
        if !self.enabled {
            return None;
        }

        for (i, diff) in self.diffs.iter().enumerate().skip(start) {
            if *diff {
                return Some(i);
            }
        }

        None
    }

    pub fn recalculate(&mut self, hex_views: &Vec<HexView>) {
        if !self.enabled {
            self.out_of_date = true;
            return;
        }

        // if !self.out_of_date {
        //     return;
        // }

        if hex_views.len() < 2 {
            self.enabled = false;
            return;
        }

        let max_size = hex_views.iter().map(|hv| hv.file.data.len()).max().unwrap();

        self.diffs = Vec::with_capacity(max_size);

        for i in 0..max_size {
            let diff = !hex_views
                .iter()
                .all(|hv| i < hv.file.data.len() && hv.file.data[i] == hex_views[0].file.data[i]);
            self.diffs.push(diff);
        }
        self.out_of_date = false;
    }
}
