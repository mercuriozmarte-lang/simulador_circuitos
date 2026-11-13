// ==========================================
// Fichero: src/main.rs
// ==========================================

// --- Bloque 1: Importaciones y Módulos ---
use crate::logica::*;
use bevy::prelude::*;
pub mod componentes;
pub mod configuracion;
pub mod lineas;
pub mod logica;

pub mod raton;

use componentes::*;
use configuracion::ConfigApp;
use lineas::*;
use raton::*;

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
                borrar_con_clic_derecho,
                limpiar_cables_huerfanos,
                // Cerebro lógico del circuito ---
                propagar_cables,
                evaluar_puertas,
                actualizar_visuales_conectores,
                // ------------------------------------------
                dibuja_linea_temporal,
                dibuja_cables_permanentes,
            )
                .chain(),
        )
        .run();
}

// Bloque: Bloque 3: Setup del Entorno
fn setup_entorno(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<ConfigApp>,
) {
    commands.spawn((Camera2d, CamaraPrincipal));

    // Creamos dos interruptores de entrada a la izquierda
    crear_interruptor(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(-300.0, 50.0),
        &config,
    );
    crear_interruptor(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(-300.0, -50.0),
        &config,
    );

    // Instanciamos una puerta AND y una OR
    crear_puerta(
        &mut commands,
        Vec2::new(-50.0, 0.0),
        TipoPuerta::And,
        &config,
    );

    crear_puerta(
        &mut commands,
        Vec2::new(150.0, 50.0),
        TipoPuerta::Or,
        &config,
    );
}
// ==========================================
// Fichero: src/main.rs
// Bloque: Bloque 4: Creador de Puertas y Conectores
// ==========================================
fn bundle_conector(
    pos: Vec3,
    tipo: TipoConector,
    config: &ConfigApp,
) -> (Sprite, Transform, TipoConector, ValorSeñal) {
    (
        Sprite {
            color: config.col_con_hueco,
            custom_size: Some(config.tam_conector),
            ..default()
        },
        Transform::from_translation(pos),
        tipo,
        ValorSeñal(false),
    )
}

fn bundle_hueco(config: &ConfigApp) -> (Sprite, Transform, CentroHueco) {
    // Cuadrado interior oscuro que hace que el conector se vea "hueco"
    (
        Sprite {
            color: Color::srgb(0.1, 0.1, 0.1),
            custom_size: Some(config.tam_conector - Vec2::splat(4.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.1),
        CentroHueco,
    )
}

fn crear_puerta(commands: &mut Commands, pos: Vec2, tipo_puerta: TipoPuerta, config: &ConfigApp) {
    // Escogemos el texto según el tipo
    let texto = match tipo_puerta {
        TipoPuerta::And => "AND",
        TipoPuerta::Or => "OR",
        TipoPuerta::Not => "NOT",
        TipoPuerta::Interruptor => "IN",
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
            padre.spawn((
                Text2d::new(texto),
                TextFont {
                    font_size: bevy::text::FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                // Z=0.2 para que el texto quede por encima del bloque
                Transform::from_xyz(0.0, 0.0, 0.2),
            ));

            // Entrada 1 (Superior izquierda)
            padre
                .spawn(bundle_conector(
                    Vec3::new(-config.tam_puerta.x / 2.0, 15.0, 0.1),
                    TipoConector::Entrada,
                    config,
                ))
                .with_children(|con| {
                    con.spawn(bundle_hueco(config));
                });

            // Entrada 2 (Inferior izquierda)
            padre
                .spawn(bundle_conector(
                    Vec3::new(-config.tam_puerta.x / 2.0, -15.0, 0.1),
                    TipoConector::Entrada,
                    config,
                ))
                .with_children(|con| {
                    con.spawn(bundle_hueco(config));
                });

            // Salida (Centro derecha)
            padre
                .spawn(bundle_conector(
                    Vec3::new(config.tam_puerta.x / 2.0, 0.0, 0.1),
                    TipoConector::Salida,
                    config,
                ))
                .with_children(|con| {
                    con.spawn(bundle_hueco(config));
                });
        });
}

// ==========================================
// Fichero: src/main.rs
// Bloque: Bloque 5: Creador de Interruptores
// ==========================================
fn crear_interruptor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec2,
    config: &ConfigApp,
) {
    // Malla triangular apuntando a la derecha
    let malla_triangulo = meshes.add(Triangle2d::new(
        Vec2::new(0.0, 15.0),
        Vec2::new(0.0, -15.0),
        Vec2::new(18.0, 0.0),
    ));
    let mat_cuerpo = materials.add(config.col_puerta);

    commands
        .spawn((
            Transform::from_xyz(pos.x, pos.y, 1.0),
            Visibility::default(),
            NodoLogico {
                tipo: TipoPuerta::Interruptor,
            },
        ))
        .with_children(|padre| {
            // 1. Primer cuadrado (Izquierda): Botón para alternar 0 (Rojo) y 1 (Verde)
            padre.spawn((
                Sprite {
                    color: config.col_estado_0,
                    custom_size: Some(Vec2::new(26.0, 26.0)),
                    ..default()
                },
                Transform::from_xyz(-28.0, 0.0, 0.1),
                BotonInterruptor { estado: false },
            ));

            // 2. Segundo cuadrado (Centro, junto al primero): Cuerpo para moverlo
            padre.spawn((
                Sprite {
                    color: config.col_puerta,
                    custom_size: Some(Vec2::new(30.0, 30.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));

            // 3. Triángulo (Derecha, pegado al segundo cuadrado): Parte del cuerpo
            padre.spawn((
                Mesh2d(malla_triangulo),
                MeshMaterial2d(mat_cuerpo),
                Transform::from_xyz(15.0, 0.0, 0.0),
            ));

            // 4. Conector de Salida en la punta del triángulo
            padre
                .spawn(bundle_conector(
                    Vec3::new(33.0, 0.0, 0.2),
                    TipoConector::Salida,
                    config,
                ))
                .with_children(|con| {
                    con.spawn(bundle_hueco(config));
                });
        });
}
