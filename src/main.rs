use rand::{Rng, thread_rng};
use servicepoint::{ByteGrid, Command, CompressionCode, Connection, FRAME_PACING, Grid, Origin, PIXEL_HEIGHT, PIXEL_WIDTH, PixelGrid, TILE_SIZE};

use crate::collision::{Point, Rect};

mod collision;

struct Nexus {
    rect: Rect,
}

struct Ant {
    position: Point,
    state: AntState,
}

type PheromoneLevel = f32;

enum AntState {
    RandomWalkFromHome(PheromoneLevel),
    CarryingFood(PheromoneLevel),
}


fn main() {
    // 172.23.42.29
    let connection = Connection::open("127.0.0.1:2342")
        .expect("connect failed");

    connection.send(Command::Brightness(255)).expect("send failed");

    let nexus = Nexus {
        rect: Rect {
            x1: PIXEL_WIDTH / 2,
            y1: PIXEL_HEIGHT / 2,
            x2: PIXEL_WIDTH / 2 + TILE_SIZE - 1,
            y2: PIXEL_HEIGHT / 2 + TILE_SIZE - 1,
        },
    };


    let mut food = ByteGrid::new(PIXEL_WIDTH, PIXEL_HEIGHT);
    for x in nexus.rect.x1 - 20..nexus.rect.x1 + 4 - 20 {
        for y in nexus.rect.y1..nexus.rect.y1 + 4 {
            food.set(x, y, 100);
        }
    }

    let mut ants = vec!();
    for _ in 0..5 {
        ants.push(Ant {
            position: nexus.rect.top_left(),
            state: AntState::RandomWalkFromHome(100f32),
        })
    }

    let mut pheromone_home = [[0f32 as PheromoneLevel; PIXEL_HEIGHT]; PIXEL_WIDTH];

    loop {
        let mut pixels = PixelGrid::max_sized();

        for ant in ants.iter_mut() {
            match ant.state {
                AntState::RandomWalkFromHome(level) => {
                    // check for food
                    let food_at = food.get(ant.position.x, ant.position.y);
                    if food_at > 0 {
                        food.set(ant.position.x, ant.position.y, food_at - 1);
                        ant.state = AntState::CarryingFood(food_at as PheromoneLevel);
                        continue;
                    }

                    ant.walk_randomly();
                    pheromone_home[ant.position.x][ant.position.y] += level;
                    ant.state = AntState::RandomWalkFromHome(level * 0.9)
                }

                AntState::CarryingFood(level) => {
                    ant.walk_randomly();
                    ant.state = AntState::CarryingFood(level * 0.9)
                }
            };

            pixels.set(ant.position.x, ant.position.y, true);
        }

        for x in 0..pixels.width() {
            for y in 0..pixels.height() {
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

        connection.send(Command::BitmapLinearWin(Origin(0, 0), pixels, CompressionCode::Uncompressed))
            .expect("send failed");
        std::thread::sleep(FRAME_PACING);
    }
}

impl Ant {
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
            _ => panic!()
        }
    }
}

fn find_pheromones(pheromones: &[[PheromoneLevel; PIXEL_HEIGHT]; PIXEL_WIDTH], position: Point) -> Option<(Point, PheromoneLevel)> {
    
}
