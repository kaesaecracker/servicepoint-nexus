use rand::{Rng, thread_rng};
use servicepoint::{ByteGrid, Grid, PIXEL_HEIGHT, PIXEL_WIDTH, RefGrid};

use crate::{collision, GridEx, Nexus, PheromoneGrid, PheromoneLevel};
use crate::collision::Point;

pub struct Ant {
    pub(crate) position: Point,
    pub(crate) state: AntState,
}

pub enum AntState {
    Searching,
    FoundFood(Point),
    Homing,
    Depositing(Point),
}

/// in pixels away from self, 0 would mean only tile currently standing on
const ANT_VISION_RANGE: isize = 1;
const ANT_SEARCHING_PHEROMONE_POSITION: f32 = 10.;
const ANT_SEARCHING_PHEROMONE_AROUND: f32 = 3.;


impl Ant {
    pub(crate) fn state_step(&mut self, food: &mut ByteGrid, home: &mut PheromoneGrid, nexus: &mut Nexus) {
        match self.state {
            AntState::Searching => {
                // check for food
                for (dx, dy) in Self::iter_vision() {
                    let x = self.position.x as isize + dx;
                    let y = self.position.y as isize + dy;

                    let food_at = match food.get_ref_mut_optional(x, y) {
                        None => continue,
                        Some(cell) => cell,
                    };

                    if *food_at > 0 {
                        self.state = AntState::FoundFood(Point {
                            x: x as usize,
                            y: y as usize,
                        });
                        return;
                    }
                }

                // walk away from home
                match find_pheromones(home, self.position, |left, right| left < right) {
                    None => self.walk_randomly(),
                    Some((position, _)) => self.position = position,
                }

                for (dx, dy) in Self::iter_vision() {
                    let cell = match home.get_optional_mut(
                        self.position.x as isize + dx,
                        self.position.y as isize + dy,
                    ) {
                        None => continue,
                        Some(cell) => cell,
                    };

                    *cell += if dx == 0 && dy == 0 {
                        ANT_SEARCHING_PHEROMONE_POSITION
                    } else {
                        ANT_SEARCHING_PHEROMONE_AROUND
                    };
                }
            }
            AntState::Homing => {
                for (dx, dy) in Self::iter_vision() {
                    let x = self.position.x.saturating_add_signed(dx);
                    let y = self.position.y.saturating_add_signed(dy);
                    let point = Point { x, y };
                    if collision::point_in_rect(point, nexus.rect) {
                        self.state = AntState::Depositing(point);
                        return;
                    }
                }

                match find_pheromones(home, self.position, |left, right| left > right) {
                    None => self.walk_randomly(),
                    Some((position, _)) => self.position = position,
                }
            }
            AntState::FoundFood(food_pos) => {
                if self.position != food_pos {
                    self.move_towards(food_pos);
                    return;
                }

                let food_on_pos = food.get(self.position.x, self.position.y);
                if food_on_pos < 1 {
                    self.state = AntState::Searching;
                    return;
                }
                food.set(self.position.x, self.position.y, food_on_pos - 1);
                self.state = AntState::Homing;
            }
            AntState::Depositing(pos) => {
                if self.position != pos {
                    self.move_towards(pos);
                    return;
                }

                nexus.food = nexus.food.saturating_add(1);
                self.state = AntState::Searching;
            }
        };
    }

    fn iter_vision() -> impl Iterator<Item=(isize, isize)> {
        let mut result = vec!();
        for dx in -ANT_VISION_RANGE..=ANT_VISION_RANGE {
            for dy in -ANT_VISION_RANGE..=ANT_VISION_RANGE {
                result.push((dx, dy));
            }
        }
        result.into_iter()
    }

    fn walk_randomly(&mut self) {
        match thread_rng().gen_range(0..4) {
            0 => {
                self.position.x = usize::clamp(self.position.x + 1, 0, PIXEL_WIDTH - 1);
            }
            1 => {
                self.position.x = self.position.x.saturating_sub(1);
            }
            2 => {
                self.position.y = usize::clamp(self.position.y + 1, 0, PIXEL_HEIGHT - 1);
            }
            3 => {
                self.position.y = self.position.y.saturating_sub(1);
            }
            _ => panic!(),
        }
    }

    fn move_towards(&mut self, food_pos: Point) {
        if food_pos.x < self.position.x {
            self.position.x -= 1;
        } else if food_pos.x > self.position.x {
            self.position.x += 1;
        } else if food_pos.y < self.position.y {
            self.position.y -= 1;
        } else if food_pos.y > self.position.y {
            self.position.y += 1;
        }
    }
}

fn find_pheromones(
    pheromones: &[[PheromoneLevel; PIXEL_HEIGHT]; PIXEL_WIDTH],
    position: Point,
    is_left: fn(PheromoneLevel, PheromoneLevel) -> bool,
) -> Option<(Point, PheromoneLevel)> {
    let mut result = None;

    for (dx, dy) in [(0, -1), (0, 1), (1, 0), (-1, 0)] {
        let x = position.x as isize + dx;
        let y = position.y as isize + dy;

        let right_level = match pheromones.get_optional(x, y) {
            None => continue,
            Some(level) => level,
        };

        let pos = Point {
            x: x as usize,
            y: y as usize,
        };
        result = match result {
            None => Some((pos, right_level)),
            Some((_, left_level)) => {
                if is_left(left_level, right_level) {
                    result
                } else {
                    Some((pos, right_level))
                }
            }
        }
    }

    result
}
