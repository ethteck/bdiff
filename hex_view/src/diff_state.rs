#[derive(Debug)]
pub struct DiffState {
    pub enabled: bool,
    pub diffs: Vec<bool>,
}

impl Default for DiffState {
    fn default() -> Self {
        Self {
            enabled: true,
            diffs: Vec::new(),
        }
    }
}

impl DiffState {
    pub fn is_diff_at(&self, index: isize) -> bool {
        if index < 0 || !self.enabled {
            return false;
        }

        let index = index as usize;

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

    pub fn recalculate(&mut self, files: &[&[u8]]) {
        if !self.enabled {
            return;
        }

        if files.len() < 2 {
            self.enabled = false;
            return;
        }

        let max_size = files.iter().map(|f| f.len()).max().unwrap();

        self.diffs = Vec::with_capacity(max_size);

        for i in 0..max_size {
            let diff = !files.iter().all(|f| i < f.len() && f[i] == files[0][i]);
            self.diffs.push(diff);
        }
    }
}
