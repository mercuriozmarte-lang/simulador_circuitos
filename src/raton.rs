// ==========================================
// Fichero: src/raton.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::componentes::*;
use crate::configuracion::ConfigApp;

// --- Bloque 2: Clic en Nodos y Conectores ---
pub fn detectar_clic_nodo(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform),
        With<CamaraPrincipal>
    >,
    q_nodos: Query<(Entity, &Transform), With<NodoLogico>>,
    q_conectores: Query<
        (Entity, &GlobalTransform, &TipoConector)
    >,
    mut estado_linea: ResMut<EstadoCrearLinea>,
    mut next_state: ResMut<NextState<ModoApp>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = q_windows.single() else { return; };
    let Ok((cam, cam_trf)) = q_camera.single() else { return; };

    if let Some(pos_pantalla) = window.cursor_position() {
        if let Ok(pos_mundo) = cam.viewport_to_world_2d(
            cam_trf,
            pos_pantalla
        ) {
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
    q_arrastrando: Query<Entity, With<Arrastrando>>,
    q_conectores: Query<
        (Entity, &GlobalTransform, &TipoConector)
    >,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform),
        With<CamaraPrincipal>
    >,
    estado_app: Res<State<ModoApp>>,
    estado_linea: Res<EstadoCrearLinea>,
    mut next_state: ResMut<NextState<ModoApp>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if !buttons.just_released(MouseButton::Left) {
        return;
    }

    for entidad in q_arrastrando.iter() {
        commands.entity(entidad).remove::<Arrastrando>();
    }

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
            if let Ok(pos_mundo) = cam.viewport_to_world_2d(
                cam_trf,
                pos_pantalla
            ) {
                for (ent_destino, g_trf, tipo) in q_conectores.iter() {
                    let pos_conector = g_trf.translation().truncate();

                    if pos_mundo.distance(pos_conector) < config.rad_clic_con {
                        if *tipo == TipoConector::Entrada {
                            if let Some(orig) = estado_linea.origen_entidad {
                                commands.spawn(Cable {
                                    origen: orig,
                                    destino: ent_destino,
                                });
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
    q_camera: Query<
        (&Camera, &GlobalTransform),
        With<CamaraPrincipal>
    >,
    mut q_arrastrando: Query<&mut Transform, With<Arrastrando>>,
) {
    if q_arrastrando.is_empty() {
        return;
    }

    let Ok(window) = q_windows.single() else { return; };
    let Ok((cam, cam_trf)) = q_camera.single() else { return; };

    if let Some(pos_pantalla) = window.cursor_position() {
        if let Ok(pos_mundo) = cam.viewport_to_world_2d(
            cam_trf,
            pos_pantalla
        ) {
            for mut trf in q_arrastrando.iter_mut() {
                trf.translation.x = pos_mundo.x;
                trf.translation.y = pos_mundo.y;
            }
        }
    }
}