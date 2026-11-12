// ==========================================
// Fichero: src/main.rs
// ==========================================

// --- Bloque 1: Importaciones y Módulos ---
use bevy::prelude::*;

pub mod componentes;
pub mod configuracion;
pub mod lineas;
pub mod raton;

use componentes::*;
use configuracion::ConfigApp;
use lineas::*;
use raton::*;

// Fichero: src/main.rs
// Bloque: Bloque 2: main
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
                // Registramos los dos nuevos sistemas de borrado
                borrar_con_clic_derecho,
                limpiar_cables_huerfanos,
                dibuja_linea_temporal,
                dibuja_cables_permanentes,
            )
                .chain(),
        )
        .run();
}
// Fichero: src/main.rs
// Bloque: Bloque 3: Setup del Entorno
fn setup_entorno(mut commands: Commands, config: Res<ConfigApp>) {
    commands.spawn((Camera2d, CamaraPrincipal));

    // Instanciamos una puerta AND
    crear_puerta(
        &mut commands,
        Vec2::new(-150.0, 0.0),
        TipoPuerta::And,
        &config,
    );

    // Instanciamos una puerta OR
    crear_puerta(
        &mut commands,
        Vec2::new(150.0, 50.0),
        TipoPuerta::Or,
        &config,
    );
}

// Fichero: src/main.rs
// Bloque: Bloque 4: Creador de Puertas
fn crear_puerta(commands: &mut Commands, pos: Vec2, tipo_puerta: TipoPuerta, config: &ConfigApp) {
    // Escogemos el texto según el tipo
    let texto = match tipo_puerta {
        TipoPuerta::And => "AND",
        TipoPuerta::Or => "OR",
        TipoPuerta::Not => "NOT",
    };

    commands
        .spawn((
            Sprite {
                color: config.col_puerta,
                custom_size: Some(config.tam_puerta),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 1.0),
            // Guardamos su tipo para evaluar la lógica en el futuro
            NodoLogico { tipo: tipo_puerta },
        ))
        .with_children(|padre| {
            // Texto central identificativo
            padre.spawn((
                Text2d::new(texto),
                TextFont {
                    font_size: bevy::text::FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 0.2),
            ));

            // Conector de Entrada 1 (Superior izquierda)
            padre.spawn((
                Sprite {
                    color: config.col_con_ent,
                    custom_size: Some(config.tam_conector),
                    ..default()
                },
                Transform::from_xyz(-config.tam_puerta.x / 2.0, 15.0, 0.1),
                TipoConector::Entrada,
            ));

            // Conector de Entrada 2 (Inferior izquierda)
            padre.spawn((
                Sprite {
                    color: config.col_con_ent,
                    custom_size: Some(config.tam_conector),
                    ..default()
                },
                Transform::from_xyz(-config.tam_puerta.x / 2.0, -15.0, 0.1),
                TipoConector::Entrada,
            ));

            // Conector de Salida (Centro derecha)
            padre.spawn((
                Sprite {
                    color: config.col_con_sal,
                    custom_size: Some(config.tam_conector),
                    ..default()
                },
                Transform::from_xyz(config.tam_puerta.x / 2.0, 0.0, 0.1),
                TipoConector::Salida,
            ));
        });
}
