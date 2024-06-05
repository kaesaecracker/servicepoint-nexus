use rand::{Rng, thread_rng};
use servicepoint::{Grid, PIXEL_HEIGHT, PIXEL_WIDTH};

use crate::{collision, Environment};
use crate::collision::Point;
use crate::nexus::Nexus;
use crate::pheromone_grid::{GridEx, PheromoneGrid, PheromoneLevel};

pub(crate) struct Ant {
    pub(crate) position: Point,
    pub(crate) state: AntState,
}

pub(crate) enum AntState {
    Searching,
    FoundFood(Point),
    Homing,
    Depositing(Point),
}

/// in pixels away from self, 0 would mean only tile currently standing on
const ANT_VISION_RANGE: isize = 1;
const ANT_SEARCHING_PHEROMONE_POSITION: f32 = 100.;
const ANT_SEARCHING_PHEROMONE_AROUND: f32 = 30.;

impl Ant {
    pub(crate) fn step(
        &mut self,
        environment: &mut Environment,
        nexus: &mut Nexus,
    ) {
        match self.state {
            AntState::Searching => {
                self.release_pheromones(&mut environment.home_pheromone);

                // check for food
                for (dx, dy) in Self::iter_vision() {
                    let x = self.position.x as isize + dx;
                    let y = self.position.y as isize + dy;

                    let food_at = match environment.food.get_ref_mut_optional(x, y) {
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
                match self.find_pheromones(&environment.home_pheromone, |left, right| left < right) {
                    None => self.walk_randomly(),
                    Some((position, _)) => self.position = position,
                }
            }
            AntState::Homing => {
                for (dx, dy) in Self::iter_vision() {
                    let point = Point {
                        x: self.position.x.saturating_add_signed(dx),
                        y: self.position.y.saturating_add_signed(dy),
                    };
                    if collision::point_in_rect(point, nexus.rect) {
                        self.state = AntState::Depositing(point);
                        return;
                    }
                }

                self.move_towards(nexus.rect.top_left);
                /*
                match self.find_pheromones(home, |left, right| left > right) {
                    None => self.walk_randomly(),
                    Some((position, _)) => self.position = position,
                }
                */
            }
            AntState::FoundFood(food_pos) => {
                self.release_pheromones(&mut environment.home_pheromone);

                if self.position != food_pos {
                    self.move_towards(food_pos);
                    return;
                }

                let food_on_pos = environment.food.get(self.position.x, self.position.y);
                if food_on_pos < 1 {
                    self.state = AntState::Searching;
                    return;
                }
                environment.food.set(self.position.x, self.position.y, food_on_pos - 1);
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
        let mut result = vec![];
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

    fn find_pheromones(
        &self,
        pheromones: &PheromoneGrid,
        is_left: fn(PheromoneLevel, PheromoneLevel) -> bool,
    ) -> Option<(Point, PheromoneLevel)> {
        let position = self.position;
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

    fn release_pheromones(&mut self, pheromone: &mut PheromoneGrid) {
        for (dx, dy) in Self::iter_vision() {
            let cell = match pheromone
                .get_optional_mut(self.position.x as isize + dx, self.position.y as isize + dy)
            {
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
}
