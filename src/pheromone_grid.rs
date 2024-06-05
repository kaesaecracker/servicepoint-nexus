use servicepoint::{PIXEL_HEIGHT, PIXEL_WIDTH};

pub(crate) type PheromoneLevel = f32;
pub(crate) type PheromoneGrid = [[PheromoneLevel; PIXEL_HEIGHT]; PIXEL_WIDTH];

pub(crate) trait GridEx<T> {
    fn get_optional(&self, x: isize, y: isize) -> Option<T>;

    fn get_optional_mut(&mut self, x: isize, y: isize) -> Option<&mut T>;

    fn get_mut(&mut self, x: usize, y: usize) -> &mut T;
}

impl GridEx<PheromoneLevel> for PheromoneGrid {
    fn get_optional(&self, x: isize, y: isize) -> Option<PheromoneLevel> {
        if x < 0 || x >= self.len() as isize || y < 0 || y >= self[0].len() as isize {
            None
        } else {
            Some(self[x as usize][y as usize])
        }
    }

    fn get_optional_mut(&mut self, x: isize, y: isize) -> Option<&mut PheromoneLevel> {
        if x < 0 || x >= self.len() as isize || y < 0 || y >= self[0].len() as isize {
            None
        } else {
            Some(&mut self[x as usize][y as usize])
        }
    }

    fn get_mut(&mut self, x: usize, y: usize) -> &mut PheromoneLevel {
        &mut self[x][y]
    }
}
