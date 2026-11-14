// ==========================================
// Fichero: src/lineas.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::componentes::*;
use crate::configuracion::ConfigApp;


// Bloque: Bloque 2: Crear Cuadrícula en el Fondo (Z = -5.0)
pub fn crear_cuadricula_fondo(
    commands: &mut Commands,
    config: &ConfigApp,
) {
    let paso = 50.0;
    let limite = 2000.0;
    let longitud = limite * 2.0;
    let grosor = 1.0;

    // Contenedor padre de toda la cuadrícula en el fondo (Z = -5.0)
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, -5.0),
        Visibility::default(),
    ))
    .with_children(|padre| {
        let mut x = -limite;
        while x <= limite {
            padre.spawn((
                Sprite {
                    color: config.col_cuadricula,
                    custom_size: Some(Vec2::new(grosor, longitud)),
                    ..default()
                },
                Transform::from_xyz(x, 0.0, 0.0),
            ));
            x += paso;
        }

        let mut y = -limite;
        while y <= limite {
            padre.spawn((
                Sprite {
                    color: config.col_cuadricula,
                    custom_size: Some(Vec2::new(longitud, grosor)),
                    ..default()
                },
                Transform::from_xyz(0.0, y, 0.0),
            ));
            y += paso;
        }
    });
}

// Bloque: Bloque 3: Dibujar Línea Temporal con Codos
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
            let mut punto_anterior = estado_linea.origen_pos;

            // Dibujamos los tramos ya fijados con codos
            for &codo in estado_linea.puntos_intermedios.iter() {
                gizmos.line_2d(
                    punto_anterior, 
                    codo, 
                    config.col_cable_tmp
                );
                gizmos.circle_2d(
                    codo, 
                    3.5, 
                    config.col_cable_tmp
                );
                punto_anterior = codo;
            }

            // Tramo elástico desde el último codo hasta el cursor actual
            gizmos.line_2d(
                punto_anterior, 
                pos_mundo, 
                config.col_cable_tmp
            );
        }
    }
}
// --- Bloque 4: Dibujar Cables Permanentes con Codos ---
pub fn dibuja_cables_permanentes(
    q_cables: Query<&Cable>,
    q_global_trf: Query<&GlobalTransform>,
    q_valores: Query<&ValorSeñal>,
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
            
            // Elegimos color verde (1) o rojo (0) según el origen
            let mut color_cable = config.col_estado_0;
            if let Ok(val) = q_valores.get(cable.origen) {
                if val.0 {
                    color_cable = config.col_estado_1;
                }
            }
            
            let mut punto_anterior = pos_origen;
            for &codo in cable.puntos_intermedios.iter() {
                gizmos.line_2d(punto_anterior, codo, color_cable);
                gizmos.circle_2d(codo, 3.0, color_cable);
                punto_anterior = codo;
            }

            gizmos.line_2d(
                punto_anterior, 
                pos_destino, 
                color_cable
            );
        }
    }
}

// --- Bloque 5: Limpiar Cables Huérfanos ---
pub fn limpiar_cables_huerfanos(
    q_cables: Query<(Entity, &Cable)>,
    q_conectores: Query<&TipoConector>,
    mut commands: Commands,
) {
    for (ent_cable, cable) in q_cables.iter() {
        let orig_existe = q_conectores.contains(cable.origen);
        let dest_existe = q_conectores.contains(cable.destino);

        if !orig_existe || !dest_existe {
            commands.entity(ent_cable).despawn();
        }
    }
}