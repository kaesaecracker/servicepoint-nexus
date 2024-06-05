use std::time::Instant;

use servicepoint::{Command, Connection, FRAME_PACING};

use crate::app::App;
use crate::environment::Environment;

mod ant;
mod collision;
mod environment;
mod app;
mod pheromone_grid;
mod nexus;

fn main() {
    // 172.23.42.29
    let connection = Connection::open("127.0.0.1:2342").expect("connect failed");

    connection
        .send(Command::Brightness(255))
        .expect("send failed");

    let mut app = App::new(connection);
    loop {
        let start = Instant::now();

        app.logic();
        app.render();

        let wait_time = (10 * FRAME_PACING) - start.elapsed();
        std::thread::sleep(wait_time);
    }
}
