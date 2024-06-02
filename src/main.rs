use rand::{Rng, thread_rng};
use servicepoint::{
    ByteGrid, Command, CompressionCode, Connection, FRAME_PACING, Grid, Origin, PIXEL_HEIGHT,
    PIXEL_WIDTH, PixelGrid, RefGrid, TILE_SIZE,
};

use crate::collision::{Point, Rect};

mod collision;

struct Nexus {
    rect: Rect,
    food: usize,
}

struct Ant {
    position: Point,
    state: AntState,
}

type PheromoneLevel = f32;

enum AntState {
    Searching,
    FoundFood(Point),
    Homing,
    Depositing,
}

/// in pixels away from self, 0 would mean only tile currently standing on
const ANT_VISION_RANGE: isize = 1;
const ANT_SEARCHING_PHEROMONE_POSITION: f32 = 10.;
const ANT_SEARCHING_PHEROMONE_AROUND: f32 = 3.;
const PHEROMONE_DIFFUSION: f32 = -0.1;

type PheromoneGrid = [[PheromoneLevel; PIXEL_HEIGHT]; PIXEL_WIDTH];

trait GridEx<T> {
    fn get_optional(&self, x: isize, y: isize) -> Option<T>;
    fn get_optional_mut(&mut self, x: isize, y: isize) -> Option<&mut T>;
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
}

fn main() {
    // 172.23.42.29
    let connection = Connection::open("127.0.0.1:2342").expect("connect failed");

    connection
        .send(Command::Brightness(255))
        .expect("send failed");

    let mut nexus = Nexus {
        rect: Rect {
            x1: PIXEL_WIDTH / 2,
            y1: PIXEL_HEIGHT / 2,
            x2: PIXEL_WIDTH / 2 + TILE_SIZE - 1,
            y2: PIXEL_HEIGHT / 2 + TILE_SIZE - 1,
        },
        food: 0,
    };

    let mut food = ByteGrid::new(PIXEL_WIDTH, PIXEL_HEIGHT);
    for x in nexus.rect.x1 - 20..nexus.rect.x1 + 4 - 20 {
        for y in nexus.rect.y1..nexus.rect.y1 + 4 {
            food.set(x, y, 100);
        }
    }

    let mut ants = vec![];
    for _ in 0..1 {
        ants.push(Ant {
            position: nexus.rect.top_left(),
            state: AntState::Searching,
        })
    }

    let mut pheromone_home: PheromoneGrid = [[0f32; PIXEL_HEIGHT]; PIXEL_WIDTH];

    loop {
        let mut pixels = PixelGrid::max_sized();

        for ant in ants.iter_mut() {
            ant.state_step(&mut food, &mut pheromone_home, &mut nexus);
            pixels.set(ant.position.x, ant.position.y, true);
        }

        for x in 0..pixels.width() {
            for y in 0..pixels.height() {
                pheromone_home[x][y] = f32::clamp(pheromone_home[x][y] - PHEROMONE_DIFFUSION, 0f32, f32::MAX);
                
                if food.get(x, y) > 0 {
                    pixels.set(x, y, true);
                }
            }
        }

        for x in nexus.rect.x1..nexus.rect.x2 {
            for y in nexus.rect.y1..nexus.rect.y2 {
                pixels.set(x, y, true);
            }
        }

        connection
            .send(Command::BitmapLinearWin(
                Origin(0, 0),
                pixels,
                CompressionCode::Uncompressed,
            ))
            .expect("send failed");
        std::thread::sleep(10 * FRAME_PACING);
    }
}

impl Ant {
    fn state_step(&mut self, food: &mut ByteGrid, home: &mut PheromoneGrid, nexus: &mut Nexus) {
        match self.state {
            AntState::Searching => {
                // check for food
                for dx in -ANT_VISION_RANGE..=ANT_VISION_RANGE {
                    for dy in -ANT_VISION_RANGE..=ANT_VISION_RANGE {
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
                }

                // walk away from home
                match find_pheromones(home, self.position, |left, right| left < right) {
                    None => self.walk_randomly(),
                    Some((position, _)) => self.position = position,
                }

                for dx in -ANT_VISION_RANGE..=ANT_VISION_RANGE {
                    for dy in -ANT_VISION_RANGE..=ANT_VISION_RANGE {
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
            }
            AntState::Homing => {
                if collision::point_in_rect(self.position, nexus.rect) {
                    self.state = AntState::Depositing;
                    return;
                }

                match find_pheromones(home, self.position, |left, right| left > right) {
                    None => self.walk_randomly(),
                    Some((position, _)) => self.position = position,
                }
            }
            AntState::FoundFood(food_pos) => {
                if food_pos.x < self.position.x {
                    self.position.x -= 1;
                } else if food_pos.x > self.position.x {
                    self.position.x += 1;
                } else if food_pos.y < self.position.y {
                    self.position.y -= 1;
                } else if food_pos.y > self.position.y {
                    self.position.y += 1;
                } else {
                    let food_on_pos = food.get(self.position.x, self.position.y);
                    if food_on_pos < 1 {
                        self.state = AntState::Searching;
                        return;
                    }
                    food.set(self.position.x, self.position.y, food_on_pos - 1);
                    self.state = AntState::Homing;
                }
            }
            AntState::Depositing => {
                nexus.food = nexus.food.saturating_add(1);
                self.state = AntState::Searching;
            }
        };
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
