// ==========================================
// Fichero: src/raton.rs
// ==========================================

use crate::componentes::*;
use crate::configuracion::ConfigApp;
use crate::{crear_interruptor, crear_led, crear_puerta};
// --- Bloque 1: Importaciones ---
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// --- Bloque 2: Clic en Menú, Nodos y Conectores ---
pub fn detectar_clic_nodo(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<CamaraPrincipal>>,
    q_nodos: Query<(Entity, &Transform), With<NodoLogico>>,
    q_conectores: Query<(Entity, &GlobalTransform, &TipoConector)>,
    // Consulta para el cuadrado interactivo del interruptor
    mut q_botones: Query<(&GlobalTransform, &mut BotonInterruptor, &mut Sprite)>,
    // NUEVO: Consultas para el menú contextual
    q_menu: Query<(Entity, &MenuContextual)>,
    q_botones_menu: Query<(&GlobalTransform, &BotonMenu)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut estado_linea: ResMut<EstadoCrearLinea>,
    mut next_state: ResMut<NextState<ModoApp>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
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
            // 0. Prioridad 0: Si el menú contextual está abierto,
            // comprobamos si se ha hecho clic en alguna opción y lo cerramos
            if let Ok((ent_menu, menu)) = q_menu.single() {
                let mut tipo_elegido = None;
                let mitad_ancho = 60.0;
                let mitad_alto = 16.0;

                for (g_trf, btn_menu) in q_botones_menu.iter() {
                    let pos_btn = g_trf.translation().truncate();
                    let dentro_x = (pos_mundo.x - pos_btn.x).abs() < mitad_ancho;
                    let dentro_y = (pos_mundo.y - pos_btn.y).abs() < mitad_alto;

                    if dentro_x && dentro_y {
                        tipo_elegido = Some(btn_menu.tipo);
                        break;
                    }
                }

                // Si eligió un botón, creamos el elemento en la cuadrícula
                if let Some(tipo) = tipo_elegido {
                    let cuadricula = 50.0;
                    let pos_snap = Vec2::new(
                        (menu.pos_creacion.x / cuadricula).round() * cuadricula,
                        (menu.pos_creacion.y / cuadricula).round() * cuadricula,
                    );

                    match tipo {
                        TipoPuerta::And | TipoPuerta::Or | TipoPuerta::Not => {
                            crear_puerta(&mut commands, pos_snap, tipo, &config);
                        }
                        TipoPuerta::Interruptor => {
                            crear_interruptor(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                pos_snap,
                                &config,
                            );
                        }
                        TipoPuerta::Led => {
                            crear_led(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                pos_snap,
                                &config,
                            );
                        }
                    }
                }

                // Cerramos siempre el menú tras el clic izquierdo
                commands.entity(ent_menu).despawn();
                return;
            }

            // 1. Prioridad 1: Clic en conectores para tirar cables
            for (ent_conector, g_trf, tipo) in q_conectores.iter() {
                let pos_conector = g_trf.translation().truncate();

                let dist = pos_mundo.distance(pos_conector);
                if dist < config.rad_clic_con {
                    if *tipo == TipoConector::Salida {
                        estado_linea.origen_entidad = Some(ent_conector);
                        estado_linea.origen_pos = pos_conector;
                        next_state.set(ModoApp::CreandoLinea);
                        return;
                    }
                }
            }

            // 2. Prioridad 2: Clic en el cuadrado de cambio de estado (0 / 1)
            for (g_trf, mut boton, mut sprite) in q_botones.iter_mut() {
                let pos_boton = g_trf.translation().truncate();
                // El cuadrado mide 26x26 (radio de clic de 13 píxeles)
                if (pos_mundo.x - pos_boton.x).abs() < 13.0
                    && (pos_mundo.y - pos_boton.y).abs() < 13.0
                {
                    boton.estado = !boton.estado;
                    if boton.estado {
                        sprite.color = config.col_estado_1;
                    } else {
                        sprite.color = config.col_estado_0;
                    }
                    // Salimos para NO arrastrar el interruptor al cambiar su estado
                    return;
                }
            }

            // 3. Prioridad 3: Clic en el resto del elemento para moverlo
            for (entidad, trf) in q_nodos.iter() {
                let centro_nodo = trf.translation.truncate();
                if pos_mundo.distance(centro_nodo) < config.rad_clic_nodo {
                    commands.entity(entidad).insert(Arrastrando);
                    return;
                }
            }
        }
    }
}

// --- Bloque 3: Soltar Ratón ---
pub fn soltar_nodo(
    buttons: Res<ButtonInput<MouseButton>>,
    // Pedimos &mut Transform para alterar la posición final
    mut q_arrastrando: Query<(Entity, &mut Transform), With<Arrastrando>>,
    q_conectores: Query<(Entity, &GlobalTransform, &TipoConector)>,
    // Consultamos los cables para evitar entradas duplicadas
    q_cables: Query<&Cable>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<CamaraPrincipal>>,
    estado_app: Res<State<ModoApp>>,
    estado_linea: Res<EstadoCrearLinea>,
    mut next_state: ResMut<NextState<ModoApp>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if !buttons.just_released(MouseButton::Left) {
        return;
    }

    // 1. Soltar nodos y aplicar el encaje magnético (Snap to Grid)
    for (entidad, mut trf) in q_arrastrando.iter_mut() {
        let cuadricula = 50.0; // El tamaño actual de nuestra celda

        // Matemáticas de redondeo para encajar en la intersección exacta
        trf.translation.x = (trf.translation.x / cuadricula).round() * cuadricula;
        trf.translation.y = (trf.translation.y / cuadricula).round() * cuadricula;

        commands.entity(entidad).remove::<Arrastrando>();
    }

    // 2. Lógica de conexión de cables
    if *estado_app.get() == ModoApp::CreandoLinea {
        let Ok(window) = q_windows.single() else {
            next_state.set(ModoApp::Edicion);
            return;
        };
        let Ok((cam, cam_trf)) = q_camera.single() else {
            next_state.set(ModoApp::Edicion);
            return;
        };

        if let Some(pos_pantalla) = window.cursor_position() {
            if let Ok(pos_mundo) = cam.viewport_to_world_2d(cam_trf, pos_pantalla) {
                for (ent_destino, g_trf, tipo) in q_conectores.iter() {
                    let pos_conector = g_trf.translation().truncate();

                    let dist = pos_mundo.distance(pos_conector);
                    if dist < config.rad_clic_con {
                        if *tipo == TipoConector::Entrada {
                            // Comprobamos si la entrada ya está ocupada
                            let entrada_ocupada = q_cables.iter().any(|c| c.destino == ent_destino);

                            if !entrada_ocupada {
                                if let Some(orig) = estado_linea.origen_entidad {
                                    commands.spawn(Cable {
                                        origen: orig,
                                        destino: ent_destino,
                                    });
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }

        next_state.set(ModoApp::Edicion);
    }
}

// --- Bloque 4: Mover Nodo Arrastrado ---
pub fn mover_nodo_arrastrado(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<CamaraPrincipal>>,
    mut q_arrastrando: Query<&mut Transform, With<Arrastrando>>,
) {
    let Ok(window) = q_windows.single() else {
        return;
    };
    let Ok((cam, cam_trf)) = q_camera.single() else {
        return;
    };

    if let Some(pos_pantalla) = window.cursor_position() {
        if let Ok(pos_mundo) = cam.viewport_to_world_2d(cam_trf, pos_pantalla) {
            for mut trf in q_arrastrando.iter_mut() {
                trf.translation.x = pos_mundo.x;
                trf.translation.y = pos_mundo.y;
            }
        }
    }
}

// --- Bloque 5: Borrar o Abrir Menú con Clic Derecho ---
pub fn borrar_con_clic_derecho(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<CamaraPrincipal>>,
    q_nodos: Query<(Entity, &Transform), With<NodoLogico>>,
    q_cables: Query<(Entity, &Cable)>,
    q_global_trf: Query<&GlobalTransform>,
    // NUEVO: Consultamos si ya hay un menú abierto
    q_menu: Query<Entity, With<MenuContextual>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }

    // Si ya había un menú abierto, lo cerramos primero
    for ent_menu in q_menu.iter() {
        commands.entity(ent_menu).despawn();
    }

    let Ok(window) = q_windows.single() else {
        return;
    };
    let Ok((cam, cam_trf)) = q_camera.single() else {
        return;
    };

    if let Some(pos_pantalla) = window.cursor_position() {
        if let Ok(pos_mundo) = cam.viewport_to_world_2d(cam_trf, pos_pantalla) {
            // 1. Intentamos borrar una puerta lógica si el ratón está encima
            for (ent_nodo, trf) in q_nodos.iter() {
                let centro = trf.translation.truncate();
                if pos_mundo.distance(centro) < config.rad_clic_nodo {
                    commands.entity(ent_nodo).despawn();
                    return;
                }
            }

            // 2. Si no tocamos puerta, comprobamos si tocamos algún cable
            for (ent_cable, cable) in q_cables.iter() {
                if let (Ok(trf_ori), Ok(trf_des)) = (
                    q_global_trf.get(cable.origen),
                    q_global_trf.get(cable.destino),
                ) {
                    let a = trf_ori.translation().truncate();
                    let b = trf_des.translation().truncate();

                    // Distancia matemática del ratón al segmento de línea A-B
                    let ab = b - a;
                    let ap = pos_mundo - a;
                    let t = (ap.dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
                    let punto_mas_cercano = a + ab * t;

                    if pos_mundo.distance(punto_mas_cercano) < 8.0 {
                        commands.entity(ent_cable).despawn();
                        return;
                    }
                }
            }

            // 3. NUEVO: Si no hemos borrado ni nodo ni cable, estamos en vacío:
            // abrimos el menú contextual de creación en este punto.
            abrir_menu_contextual(&mut commands, pos_mundo);
        }
    }
}

// --- Bloque 6: Creador del Menú Contextual (NUEVO) ---
fn abrir_menu_contextual(commands: &mut Commands, pos: Vec2) {
    let opciones = [
        ("Puerta AND", TipoPuerta::And),
        ("Puerta OR", TipoPuerta::Or),
        ("Puerta NOT", TipoPuerta::Not),
        ("Interruptor", TipoPuerta::Interruptor),
        ("LED Salida", TipoPuerta::Led),
    ];

    let ancho_btn = 120.0;
    let alto_btn = 32.0;
    let separacion = 36.0;
    let total_alto = (opciones.len() as f32) * separacion + 10.0;

    // Contenedor del menú en Z = 10.0 para quedar por encima del circuito
    commands
        .spawn((
            Sprite {
                color: Color::srgb(0.12, 0.12, 0.15),
                custom_size: Some(Vec2::new(ancho_btn + 12.0, total_alto)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 10.0),
            MenuContextual { pos_creacion: pos },
        ))
        .with_children(|padre| {
            let inicio_y = ((opciones.len() as f32 - 1.0) * separacion) / 2.0;

            for (i, (texto, tipo)) in opciones.iter().enumerate() {
                let y = inicio_y - (i as f32 * separacion);

                // Botón de cada opción
                padre
                    .spawn((
                        Sprite {
                            color: Color::srgb(0.25, 0.28, 0.35),
                            custom_size: Some(Vec2::new(ancho_btn, alto_btn)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, y, 0.1),
                        BotonMenu { tipo: *tipo },
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text2d::new(*texto),
                            TextFont {
                                font_size: bevy::text::FontSize::Px(15.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Transform::from_xyz(0.0, 0.0, 0.1),
                        ));
                    });
            }
        });
}
