// ==========================================
// Fichero: src/main.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// --- Bloque 2: Estados y Marcadores ---
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum ModoApp {
    #[default]
    Edicion,
    CreandoLinea,
}

// Bloque: Bloque 2.1: Conectores (NUEVO)
#[derive(Component, PartialEq, Eq)]
pub enum TipoConector {
    Entrada,
    Salida,
}

// Bloque: Bloque 2.2: Recursos
// Recurso para recordar desde dónde estamos tirando la línea
#[derive(Resource, Default)]
pub struct EstadoCrearLinea {
    pub origen_entidad: Option<Entity>,
    pub origen_pos: Vec2,
}

// Marcador permanente para nodos lógicos
#[derive(Component)]
pub struct NodoLogico;

// Marcador TEMPORAL para el nodo que movemos
#[derive(Component)]
pub struct Arrastrando;

// Necesario para convertir coordenadas pantalla->mundo
#[derive(Component)]
pub struct CamaraPrincipal;

// Bloque: Bloque 3: main
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
        .init_state::<ModoApp>()
        // Inicializamos nuestro nuevo recurso de líneas
        .init_resource::<EstadoCrearLinea>()
        .add_systems(Startup, setup_entorno)
        .add_systems(
            Update,
            (
                dibuja_cuadricula,
                detectar_clic_nodo,
                soltar_nodo,
                mover_nodo_arrastrado,
                // Añadimos el nuevo sistema para pintar el cable
                dibuja_linea_temporal,
            )
                .chain(),
        )
        .run();
}

// Bloque: Bloque 4: Setup
fn setup_entorno(mut commands: Commands) {
    commands.spawn((Camera2d, CamaraPrincipal));

    // Puerta lógica base (PADRE)
    commands
        .spawn((
            Sprite {
                color: Color::srgb(0.2, 0.6, 0.8),
                custom_size: Some(Vec2::new(80.0, 50.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
            NodoLogico,
        ))
        // Añadimos los conectores como HIJOS
        .with_children(|padre| {
            // Conector de Entrada (Izquierda - Verde)
            padre.spawn((
                Sprite {
                    color: Color::srgb(0.2, 0.8, 0.2),
                    custom_size: Some(Vec2::new(12.0, 12.0)),
                    ..default()
                },
                // Posición relativa al centro del padre (-40 en X)
                Transform::from_xyz(-40.0, 0.0, 0.1),
                TipoConector::Entrada,
            ));

            // Conector de Salida (Derecha - Rojo)
            padre.spawn((
                Sprite {
                    color: Color::srgb(0.8, 0.2, 0.2),
                    custom_size: Some(Vec2::new(12.0, 12.0)),
                    ..default()
                },
                // Posición relativa al centro del padre (+40 en X)
                Transform::from_xyz(40.0, 0.0, 0.1),
                TipoConector::Salida,
            ));
        });
}
// --- Bloque 5: Cuadrícula ---
fn dibuja_cuadricula(mut gizmos: Gizmos) {
    let color = Color::srgb(0.2, 0.2, 0.2);
    for i in -10..=10 {
        let pos = i as f32 * 50.0;
        gizmos.line_2d(Vec2::new(pos, -500.0), Vec2::new(pos, 500.0), color);
        gizmos.line_2d(Vec2::new(-500.0, pos), Vec2::new(500.0, pos), color);
    }
}

// Fichero: src/main.rs
// Bloque: Bloque 6: Lógica de clics en el mundo 2D
fn detectar_clic_nodo(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<CamaraPrincipal>>,
    q_nodos: Query<(Entity, &Transform), With<NodoLogico>>,
    // NUEVO: Consultamos los conectores y su posición global
    q_conectores: Query<(Entity, &GlobalTransform, &TipoConector)>,
    mut estado_linea: ResMut<EstadoCrearLinea>,
    mut next_state: ResMut<NextState<ModoApp>>,
    mut commands: Commands,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = q_windows.single() else {
        return;
    };
    let Ok((cam, cam_trf)) = q_camera.single() else {
        return;
    };

    if let Some(pos_pantalla) = window.cursor_position() {
        if let Ok(pos_mundo) = cam.viewport_to_world_2d(cam_trf, pos_pantalla) {
            // 1. PRIORIDAD: ¿Hemos clicado un conector?
            for (ent_conector, g_trf, tipo) in q_conectores.iter() {
                let pos_conector = g_trf.translation().truncate();
                // Distancia pequeña para ser precisos (12 píxeles)
                if pos_mundo.distance(pos_conector) < 12.0 {
                    if *tipo == TipoConector::Salida {
                        estado_linea.origen_entidad = Some(ent_conector);
                        estado_linea.origen_pos = pos_conector;
                        // Cambiamos el estado a dibujar
                        next_state.set(ModoApp::CreandoLinea);
                        return;
                    }
                }
            }

            // 2. Si no es un conector, intentamos mover el nodo padre
            for (entidad, trf) in q_nodos.iter() {
                let centro_nodo = trf.translation.truncate();
                if pos_mundo.distance(centro_nodo) < 40.0 {
                    commands.entity(entidad).insert(Arrastrando);
                    return;
                }
            }
        }
    }
}

// Fichero: src/main.rs
// Bloque: Bloque 7: Soltar el nodo
fn soltar_nodo(
    buttons: Res<ButtonInput<MouseButton>>,
    q_arrastrando: Query<Entity, With<Arrastrando>>,
    estado_app: Res<State<ModoApp>>,
    mut next_state: ResMut<NextState<ModoApp>>,
    mut commands: Commands,
) {
    if buttons.just_released(MouseButton::Left) {
        // Quitamos la pegatina si estábamos arrastrando un nodo
        for entidad in q_arrastrando.iter() {
            commands.entity(entidad).remove::<Arrastrando>();
        }

        // Si estábamos tirando un cable, volvemos a edición normal
        // (En la Fase 5 verificaremos aquí si conectó en un destino)
        if *estado_app.get() == ModoApp::CreandoLinea {
            next_state.set(ModoApp::Edicion);
        }
    }
}

// Bloque: Bloque 8: Mover el nodo
fn mover_nodo_arrastrado(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<CamaraPrincipal>>,
    mut q_arrastrando: Query<&mut Transform, With<Arrastrando>>,
) {
    // Si no hay nodos con la pegatina, ahorramos CPU
    if q_arrastrando.is_empty() {
        return;
    }

    // Extracción segura
    let Ok(window) = q_windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = q_camera.single() else {
        return;
    };

    if let Some(pos_pantalla) = window.cursor_position() {
        if let Ok(pos_mundo) = camera.viewport_to_world_2d(camera_transform, pos_pantalla) {
            for mut trf in q_arrastrando.iter_mut() {
                // El centro del nodo persigue a la flecha del ratón
                trf.translation.x = pos_mundo.x;
                trf.translation.y = pos_mundo.y;
            }
        }
    }
}

// Bloque: Bloque 9: Dibujar línea temporal
fn dibuja_linea_temporal(
    estado_app: Res<State<ModoApp>>,
    estado_linea: Res<EstadoCrearLinea>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<CamaraPrincipal>>,
    mut gizmos: Gizmos,
) {
    // Si no estamos en modo dibujar, no gastamos CPU
    if *estado_app.get() != ModoApp::CreandoLinea {
        return;
    }

    let Ok(window) = q_windows.single() else {
        return;
    };
    let Ok((cam, cam_trf)) = q_camera.single() else {
        return;
    };

    if let Some(pos_pantalla) = window.cursor_position() {
        if let Ok(pos_mundo) = cam.viewport_to_world_2d(cam_trf, pos_pantalla) {
            // Gizmo dibuja una bonita línea blanca
            // desde el conector hasta el ratón
            gizmos.line_2d(estado_linea.origen_pos, pos_mundo, Color::WHITE);
        }
    }
}
