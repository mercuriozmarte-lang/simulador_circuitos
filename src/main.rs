// ==========================================
// Fichero: src/main.rs
// ==========================================

// --- Bloque 1: Importaciones y Módulos ---
use bevy::prelude::*;

pub mod configuracion;
pub mod componentes;
pub mod raton;
pub mod lineas;

use configuracion::ConfigApp;
use componentes::*;
use raton::*;
use lineas::*;

// --- Bloque 2: main ---
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Simulador Lógico Moderno".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        // Inyectamos la configuración
        .init_resource::<ConfigApp>()
        .init_resource::<EstadoCrearLinea>()
        .init_state::<ModoApp>()
        .add_systems(Startup, setup_entorno)
        .add_systems(
            Update,
            (
                dibuja_cuadricula,
                detectar_clic_nodo,
                soltar_nodo,
                mover_nodo_arrastrado,
                dibuja_linea_temporal,
                dibuja_cables_permanentes
            ).chain()
        )
        .run();
}

// --- Bloque 3: Setup del Entorno ---
fn setup_entorno(
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    commands.spawn((
        Camera2d,
        CamaraPrincipal
    ));

    crear_puerta(&mut commands, Vec2::new(-150.0, 0.0), &config);
    crear_puerta(&mut commands, Vec2::new(150.0, 50.0), &config);
}

// --- Bloque 4: Creador de Puertas ---
fn crear_puerta(
    commands: &mut Commands,
    pos: Vec2,
    config: &ConfigApp,
) {
    commands.spawn((
        Sprite {
            color: config.col_puerta,
            custom_size: Some(config.tam_puerta),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 1.0),
        NodoLogico,
    ))
        .with_children(|padre| {

            // Conector Entrada (se ubica automáticamente en el borde)
            padre.spawn((
                Sprite {
                    color: config.col_con_ent,
                    custom_size: Some(config.tam_conector),
                    ..default()
                },
                Transform::from_xyz(
                    -config.tam_puerta.x / 2.0,
                    0.0,
                    0.1
                ),
                TipoConector::Entrada,
            ));

            // Conector Salida
            padre.spawn((
                Sprite {
                    color: config.col_con_sal,
                    custom_size: Some(config.tam_conector),
                    ..default()
                },
                Transform::from_xyz(
                    config.tam_puerta.x / 2.0,
                    0.0,
                    0.1
                ),
                TipoConector::Salida,
            ));
        });
}