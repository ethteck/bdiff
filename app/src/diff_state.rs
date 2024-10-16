use crate::file_view::FileView;

#[derive(Debug, Default)]
pub struct DiffState {
    pub diffs: Vec<bool>,
}

impl DiffState {
    pub fn get_next_diff(&self, start: usize) -> Option<usize> {
        for (i, diff) in self.diffs.iter().enumerate().skip(start) {
            if *diff {
                return Some(i);
            }
        }

        None
    }

    pub fn recalculate(&mut self, file_views: &[FileView]) {
        if file_views.len() < 2 {
            return;
        }

        let max_size = file_views
            .iter()
            .map(|fv| fv.file.data.len() + fv.cur_pos)
            .max()
            .unwrap();

        self.diffs = Vec::with_capacity(max_size);

        for i in 0..max_size {
            let comps: Vec<u8> = file_views
                .iter()
                .filter(|fv| i >= fv.cur_pos && i < fv.cur_pos + fv.file.data.len())
                .map(|fv| fv.file.data[(i as isize - fv.cur_pos as isize) as usize])
                .collect();

            if comps.len() < 2 {
                self.diffs.push(false);
            } else {
                let first = comps[0];
                let same = comps.iter().all(|&c| c == first);
                self.diffs.push(!same);
            }
        }
    }
}
