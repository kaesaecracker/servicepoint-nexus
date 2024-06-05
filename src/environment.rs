use servicepoint::{ByteGrid, Grid, PIXEL_HEIGHT, PIXEL_WIDTH, PixelGrid};
use crate::pheromone_grid::{GridEx, PheromoneGrid};

const PHEROMONE_DIFFUSION: f32 = 0.05;
pub(crate) struct Environment {
    pub(crate) food: ByteGrid,
    pub(crate) food_pheromone: PheromoneGrid,
    pub(crate) home_pheromone: PheromoneGrid,
}

impl Environment {
    pub(crate) fn render(&self, pixels: &mut PixelGrid) {
        for x in 0..pixels.width() {
            for y in 0..pixels.height() {
                if self.food.get(x, y) > 0 {
                    pixels.set(x, y, true);
                }
            }
        }
    }

    pub(crate) fn step(&mut self) {
        Self::diffuse(&mut self.food_pheromone);
        Self::diffuse(&mut self.home_pheromone);
    }

    fn diffuse(pheromone: &mut PheromoneGrid) {
        for x in 0..PIXEL_WIDTH {
            for y in 0..PIXEL_HEIGHT {
                let cell = pheromone.get_mut(x, y);
                let amount = PHEROMONE_DIFFUSION * *cell;
                if amount <= 0f32 {
                    continue;
                }

                *cell -= amount;
                for (dx, dy) in [(0, -1), (0, 1), (1, 0), (-1, 0)] {
                    match pheromone.get_optional_mut(x as isize + dx, y as isize + dy) {
                        None => {}
                        Some(cell) => {
                            *cell += amount / 4f32;
                        }
                    }
                }
            }
        }
    }
}
