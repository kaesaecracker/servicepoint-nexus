use servicepoint::{ByteGrid, Command, CompressionCode, Connection, Grid, Origin, PIXEL_HEIGHT, PIXEL_WIDTH, PixelGrid, TILE_SIZE};

use crate::ant::{Ant, AntState};
use crate::collision::{Point, Rect};
use crate::environment::Environment;
use crate::nexus::Nexus;

pub(crate) struct App {
    nexus: Nexus,
    ants: Vec<Ant>,
    environment: Environment,
    connection: Connection,
}

impl App {
    pub(crate) fn new(connection: Connection) -> App {
        let mut app = App {
            connection,
            nexus: Nexus {
                rect: Rect {
                    top_left: Point {
                        x: PIXEL_WIDTH / 2,
                        y: PIXEL_HEIGHT / 2,

                    },
                    bottom_right: Point {
                        x: PIXEL_WIDTH / 2 + TILE_SIZE - 1,
                        y: PIXEL_HEIGHT / 2 + TILE_SIZE - 1,
                    },
                },
                food: 0,
            },
            ants: vec![],
            environment: Environment {
                food: ByteGrid::new(PIXEL_WIDTH, PIXEL_HEIGHT),
                food_pheromone: [[0f32; PIXEL_HEIGHT]; PIXEL_WIDTH],
                home_pheromone: [[0f32; PIXEL_HEIGHT]; PIXEL_WIDTH],
            },
        };

        // add some test food
        for x in app.nexus.rect.top_left.x - 20..app.nexus.rect.top_left.x + 4 - 20 {
            for y in app.nexus.rect.top_left.y..app.nexus.rect.top_left.y + 4 {
                app.environment.food.set(x, y, 100);
            }
        }

        // add some test ants
        for _ in 0..1000 {
            app.ants.push(Ant {
                position: app.nexus.rect.top_left,
                state: AntState::Searching,
            })
        }

        app
    }

    pub(crate) fn logic(&mut self) {
        self.environment.step();

        for ant in self.ants.iter_mut() {
            ant.step(&mut self.environment, &mut self.nexus);
        }
    }

    pub(crate) fn render(&mut self) {
        let mut pixels = PixelGrid::max_sized();

        for ant in self.ants.iter() {
            pixels.set(ant.position.x, ant.position.y, true);
        }

        self.environment.render(&mut pixels);
        self.nexus.render(&mut pixels);

        self.connection
            .send(Command::BitmapLinearWin(
                Origin(0, 0),
                pixels,
                CompressionCode::Uncompressed,
            ))
            .expect("send failed");
    }
}
