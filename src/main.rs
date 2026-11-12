use bevy::prelude::*;

// 1. Definimos los estados de nuestra aplicación de forma nativa
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum ModoApp {
    #[default]
    Edicion,       // Mover nodos, clics normales
    CreandoLinea,  // Modo dibujo de cables
}

// Fichero: src/main.rs
// Bloque: main
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Simulador Lógico Moderno".into(),
                // Sin los puntos decimales para que sean u32
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        // Inicializamos la máquina de estados
        .init_state::<ModoApp>()
        // Sistema de inicio puro
        .add_systems(Startup, setup_entorno)
        // Sistema de renderizado continuo (ej: cuadrícula)
        .add_systems(Update, dibuja_cuadricula)
        .run();
}

fn setup_entorno(mut commands: Commands) {
    // La cámara principal
    commands.spawn(Camera2d);
    // Aquí cargaremos fuentes e imágenes más adelante
}

fn dibuja_cuadricula(mut gizmos: Gizmos) {
    // Gizmos es nativo y ultra-rápido para líneas de debug y cuadrículas
    let color = Color::srgb(0.2, 0.2, 0.2);
    for i in -10..=10 {
        let pos = i as f32 * 50.0;
        // Lineas verticales y horizontales
        gizmos.line_2d(Vec2::new(pos, -500.0), Vec2::new(pos, 500.0), color);
        gizmos.line_2d(Vec2::new(-500.0, pos), Vec2::new(500.0, pos), color);
    }
}