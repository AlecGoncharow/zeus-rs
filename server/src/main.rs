use core::message::{Complex, GameMessage};
use hermes::tokio;
use hermes::Message;
use hermes::ServerInterface;

use core::entity::cube::Cuboid;
use core::entity::sun::Sun;
use core::entity::EntityKind;
use core::Color;

use core::proc_gen;
use proc_gen::color::ColorGenerator;
use proc_gen::noise::Perlin;
use proc_gen::terrain::TerrainGenerator;

mod entity_manager;
use entity_manager::EntityManager;

struct ServerState {
    entity_manager: EntityManager,
    id_counter: usize,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            id_counter: 1,
            entity_manager: EntityManager::new(),
        }
    }
}

fn generate_cubes(state: &mut ServerState) {
    let cube = Cuboid::cube(5.0, (0, 0, 0).into(), None, None);
    state.entity_manager.push_entity(EntityKind::from(cube));

    let sun = Sun::new(
        (0, 10, 0).into(),
        5.0,
        Color::new(255, 250, 209),
        Color::new(255, 250, 209),
    );
    state.entity_manager.push_entity(EntityKind::from(sun));

    /*
    let cube = Cuboid::cube(1.0, (10, 0, 10).into(), None);
    state.entity_manager.push_entity(EntityKind::from(cube));

    let cube = Cuboid::cube(1.0, (0, 0, 10).into(), None);
    state.entity_manager.push_entity(EntityKind::from(cube));

    let cube = Cuboid::cube(1.0, (10, 0, 0).into(), None);
    state.entity_manager.push_entity(EntityKind::from(cube));

    let cube = Cuboid::cube(1.0, (10, 10, 10).into(), None);
    state.entity_manager.push_entity(EntityKind::from(cube));

    let cube = Cuboid::cube(1.0, (0, 10, 10).into(), None);
    state.entity_manager.push_entity(EntityKind::from(cube));

    let cube = Cuboid::cube(1.0, (10, 10, 0).into(), None);
    state.entity_manager.push_entity(EntityKind::from(cube));

    let cube = Cuboid::cube(1.0, (0, 10, 0).into(), None);
    state.entity_manager.push_entity(EntityKind::from(cube));

    let cube = Cuboid::cube(5.0, (5, 5, 5).into(), None);
    state.entity_manager.push_entity(EntityKind::from(cube));

    */
    //let cube = Cuboid::cube(100.0, (0, -105, 0).into(), None);
    //state.entity_manager.push_entity(EntityKind::from(cube));
}

fn generate_terrain(seed: Option<isize>) -> core::entity::terrain::Terrain {
    let mut perlin = Perlin::default();
    if let Some(seed) = seed {
        perlin.seed = seed;
    }

    let color_gen = ColorGenerator::new(
        vec![
            (201, 178, 99).into(),
            (135, 184, 82).into(),
            (80, 171, 93).into(),
            (120, 120, 120).into(),
            (200, 200, 210).into(),
        ],
        0.45,
    );

    let terrain_gen = TerrainGenerator::new(perlin, color_gen);
    let terrain_size = if cfg!(debug_assertions) { 10 } else { 250 };
    let mut terrain = terrain_gen.generate(terrain_size, false);
    terrain.center = (terrain_size as f32 / 2., 0., terrain_size as f32 / 2.).into();

    terrain
}

#[tokio::main]
async fn main() {
    let mut state = ServerState::new();
    let mut server: ServerInterface<GameMessage> = ServerInterface::new(8080);
    server.start().await;
    let mut connection_count: usize = 0;

    generate_cubes(&mut state);

    loop {
        //server.update().await;
        let curr_connection_count: usize = server.connection_count();
        if connection_count != curr_connection_count {
            println!(
                "[Driver] change in connection count: old:{} new:{}",
                connection_count, curr_connection_count
            );
            let ping = Message::new(GameMessage::Ping);
            server.send_to_all(ping).await;
            connection_count = curr_connection_count;
        }

        if let Some((client_id, mut msg)) = server.pop_message() {
            println!("popped msg: {:?}", msg.header);
            match msg.header.id {
                GameMessage::GetId => {
                    let id = state.id_counter;
                    msg.push(id);
                    server.send_to(client_id, msg).await;
                    state.id_counter += 1;
                }
                GameMessage::SyncWorld => {
                    println!(
                        "[SyncWorld] Entities count: {:#?}",
                        state.entity_manager.entities.len()
                    );

                    for entity in state.entity_manager.entities.iter().copied() {
                        msg.push(entity);
                    }
                    println!("[SyncWorld] Final msg header {:#?}", msg.header);

                    server.send_to(client_id, msg).await;
                }
                GameMessage::RegenerateTerrain(_) => {
                    println!("[RegenerateTerrain] regenerating terrain");
                }
                GameMessage::Player => {}
                GameMessage::Ping => {}
                GameMessage::Interact => {}
                GameMessage::MovePlayer => {
                    let parse: Complex = msg.pull().unwrap();
                    println!("parsed bytes for MovePlayer: {:#?}", parse);
                }
            }
        }
    }
}
