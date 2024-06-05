use servicepoint::{Grid, PixelGrid};

use crate::collision::Rect;

pub(crate) struct Nexus {
    pub(crate) rect: Rect,
    pub(crate) food: usize,
}

impl Nexus {
    pub fn render(&self, pixel_grid: &mut PixelGrid) {
        for x in self.rect.top_left.x..self.rect.bottom_right.x {
            for y in self.rect.top_left.y..self.rect.bottom_right.y {
                pixel_grid.set(x, y, true);
            }
        }
    }
}

