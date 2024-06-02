use servicepoint::{
    ByteGrid, Command, CompressionCode, Connection, FRAME_PACING, Grid, Origin, PIXEL_HEIGHT,
    PIXEL_WIDTH, PixelGrid, TILE_SIZE,
};

use crate::ant::{Ant, AntState};
use crate::collision::Rect;

mod collision;
mod ant;

struct Nexus {
    rect: Rect,
    food: usize,
}

type PheromoneLevel = f32;

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