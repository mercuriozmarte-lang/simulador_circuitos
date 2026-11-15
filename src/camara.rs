// ==========================================
// Fichero: src/camara.rs
// --- Bloque 1: Importaciones ---
// ==========================================
use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use crate::componentes::*;

// ==========================================
// Fichero: src/camara.rs
// Bloque: Bloque 2: Sistema de Movimiento (Pan) y Zoom en Lienzo Infinito
// ==========================================
pub fn controlar_camara(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut ev_motion: MessageReader<MouseMotion>,
    mut ev_scroll: MessageReader<MouseWheel>,
    q_dialogos: Query<
        Entity, 
        Or<(With<DialogoArchivo>, With<DialogoEtiquetaPin>)>
    >,
    q_arrastrando: Query<Entity, With<Arrastrando>>,
    mut q_cam: Query<&mut Transform, With<CamaraPrincipal>>,
) {
    let Ok(mut cam_trf) = q_cam.single_mut() else {
        return;
    };

    let mut delta_raton = Vec2::ZERO;
    for ev in ev_motion.read() {
        delta_raton += ev.delta;
    }

    let arrastrando_lienzo = buttons.pressed(MouseButton::Middle)
        || (keys.pressed(KeyCode::Space) && buttons.pressed(MouseButton::Left));

    if arrastrando_lienzo {
        let escala_actual = cam_trf.scale.x;
        cam_trf.translation.x -= delta_raton.x * escala_actual;
        cam_trf.translation.y += delta_raton.y * escala_actual;
    }

    if q_dialogos.is_empty() {
        let vel_pan = 500.0 * time.delta_secs() * cam_trf.scale.x;
        if keys.pressed(KeyCode::ArrowLeft) {
            cam_trf.translation.x -= vel_pan;
        }
        if keys.pressed(KeyCode::ArrowRight) {
            cam_trf.translation.x += vel_pan;
        }
        // Si no estamos llevando piezas pegadas al cursor, Arriba/Abajo mueven la cámara
        if q_arrastrando.is_empty() {
            if keys.pressed(KeyCode::ArrowUp) {
                cam_trf.translation.y += vel_pan;
            }
            if keys.pressed(KeyCode::ArrowDown) {
                cam_trf.translation.y -= vel_pan;
            }
        }
        if keys.just_pressed(KeyCode::Home) {
            cam_trf.translation.x = 0.0;
            cam_trf.translation.y = 0.0;
            cam_trf.scale = Vec3::ONE;
        }
    }

    for ev in ev_scroll.read() {
        if ev.y.abs() > 0.001 {
            let factor = if ev.y > 0.0 { 0.90 } else { 1.10 };
            let nueva_escala = (cam_trf.scale.x * factor).clamp(0.25, 4.0);
            cam_trf.scale = Vec3::new(nueva_escala, nueva_escala, 1.0);
        }
    }
}