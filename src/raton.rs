// ==========================================
// Fichero: src/raton.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use crate::componentes::*;
use crate::configuracion::ConfigApp;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// Bloque: Bloque 2: Clic en Nodos y Conectores
pub fn detectar_clic_nodo(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<CamaraPrincipal>>,
    q_nodos: Query<(Entity, &Transform), With<NodoLogico>>,
    q_conectores: Query<(Entity, &GlobalTransform, &TipoConector)>,
    // NUEVO: Consulta para el cuadrado interactivo del interruptor
    mut q_botones: Query<(&GlobalTransform, &mut BotonInterruptor, &mut Sprite)>,
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
            // 1. Prioridad 1: Clic en conectores para tirar cables
            for (ent_conector, g_trf, tipo) in q_conectores.iter() {
                let pos_conector = g_trf.translation().truncate();

                if pos_mundo.distance(pos_conector) < config.rad_clic_con {
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

// Bloque: Bloque 3: Soltar Ratón
pub fn soltar_nodo(
    buttons: Res<ButtonInput<MouseButton>>,
    // Pedimos &mut Transform para alterar la posición final
    mut q_arrastrando: Query<(Entity, &mut Transform), With<Arrastrando>>,
    q_conectores: Query<(Entity, &GlobalTransform, &TipoConector)>,
    // NUEVO: Consultamos los cables para evitar entradas duplicadas
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

// --- Bloque 4: Arrastrar Nodo ---
pub fn mover_nodo_arrastrado(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<CamaraPrincipal>>,
    mut q_arrastrando: Query<&mut Transform, With<Arrastrando>>,
) {
    if q_arrastrando.is_empty() {
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
            for mut trf in q_arrastrando.iter_mut() {
                trf.translation.x = pos_mundo.x;
                trf.translation.y = pos_mundo.y;
            }
        }
    }
}

// Fichero: src/raton.rs
// Bloque: Bloque 5: Funciones auxiliares y borrado
pub fn dist_punto_segmento(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let len_sq = ab.length_squared();

    // Si el origen y destino son el mismo punto
    if len_sq == 0.0 {
        return p.distance(a);
    }

    let proj = ap.dot(ab) / len_sq;
    // Limitamos la proyección entre 0 y 1 (dentro del segmento)
    let t = proj.clamp(0.0, 1.0);
    let punto_cercano = a + ab * t;

    p.distance(punto_cercano)
}

pub fn borrar_con_clic_derecho(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<CamaraPrincipal>>,
    q_nodos: Query<(Entity, &Transform), With<NodoLogico>>,
    q_cables: Query<(Entity, &Cable)>,
    q_global_trf: Query<&GlobalTransform>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
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
            // 1. Prioridad 1: Borrar nodos (puertas lógicas)
            for (entidad, trf) in q_nodos.iter() {
                let centro_nodo = trf.translation.truncate();
                if pos_mundo.distance(centro_nodo) < config.rad_clic_nodo {
                    // Magia moderna de Bevy: despawn() ahora borra
                    // automáticamente la puerta Y sus hijos (conectores)
                    commands.entity(entidad).despawn();
                    return;
                }
            }

            // 2. Prioridad 2: Borrar cables
            for (ent_cable, cable) in q_cables.iter() {
                if let (Ok(trf_ori), Ok(trf_des)) = (
                    q_global_trf.get(cable.origen),
                    q_global_trf.get(cable.destino),
                ) {
                    let p_ori = trf_ori.translation().truncate();
                    let p_des = trf_des.translation().truncate();

                    // Si hacemos clic a menos de 10 píxeles del cable
                    if dist_punto_segmento(pos_mundo, p_ori, p_des) < 10.0 {
                        commands.entity(ent_cable).despawn();
                        return;
                    }
                }
            }
        }
    }
}
