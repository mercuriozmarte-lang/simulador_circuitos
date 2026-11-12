// ==========================================
// Fichero: src/lineas.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::componentes::*;
use crate::configuracion::ConfigApp;

// --- Bloque 2: Dibujar Cuadrícula ---
pub fn dibuja_cuadricula(
    mut gizmos: Gizmos,
    config: Res<ConfigApp>,
) {
    let color = config.col_cuadricula;
    for i in -10..=10 {
        let pos = i as f32 * 50.0;
        gizmos.line_2d(
            Vec2::new(pos, -500.0),
            Vec2::new(pos, 500.0),
            color
        );
        gizmos.line_2d(
            Vec2::new(-500.0, pos),
            Vec2::new(500.0, pos),
            color
        );
    }
}

// --- Bloque 3: Dibujar Línea Temporal ---
pub fn dibuja_linea_temporal(
    estado_app: Res<State<ModoApp>>,
    estado_linea: Res<EstadoCrearLinea>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform),
        With<CamaraPrincipal>
    >,
    mut gizmos: Gizmos,
    config: Res<ConfigApp>,
) {
    if *estado_app.get() != ModoApp::CreandoLinea {
        return;
    }

    let Ok(window) = q_windows.single() else { return; };
    let Ok((cam, cam_trf)) = q_camera.single() else { return; };

    if let Some(pos_pantalla) = window.cursor_position() {
        if let Ok(pos_mundo) = cam.viewport_to_world_2d(
            cam_trf,
            pos_pantalla
        ) {
            gizmos.line_2d(
                estado_linea.origen_pos,
                pos_mundo,
                config.col_cable_tmp
            );
        }
    }
}

// --- Bloque 4: Dibujar Cables Fijos ---
pub fn dibuja_cables_permanentes(
    q_cables: Query<&Cable>,
    q_global_trf: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
    config: Res<ConfigApp>,
) {
    for cable in q_cables.iter() {
        if let (Ok(trf_ori), Ok(trf_des)) = (
            q_global_trf.get(cable.origen),
            q_global_trf.get(cable.destino)
        ) {
            let pos_origen = trf_ori.translation().truncate();
            let pos_destino = trf_des.translation().truncate();

            gizmos.line_2d(
                pos_origen,
                pos_destino,
                config.col_cable_fij
            );
        }
    }
}