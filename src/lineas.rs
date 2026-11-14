// ==========================================
// Fichero: src/lineas.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::componentes::*;
use crate::configuracion::ConfigApp;

// ==========================================
// Fichero: src/lineas.rs
// Bloque: Bloque 2: Crear Cuadrícula Infinita y Configurar Grosor de Cables
// ==========================================
pub fn crear_cuadricula_fondo(
    commands: &mut Commands,
    config_store: &mut ResMut<GizmoConfigStore>,
    config: &ConfigApp,
) {
    // Aplicamos el grosor de línea configurado en ConfigApp a todos los cables (Gizmos)
    let (gizmo_config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    gizmo_config.line.width = config.grosor_cable;

    let paso = 50.0;
    // Radio amplio (4500px) para cubrir toda la pantalla incluso con el zoom alejado al máximo
    let limite = 4500.0;
    let longitud = limite * 2.0;
    let grosor = 1.0;

    commands.spawn((
        Transform::from_xyz(0.0, 0.0, -5.0),
        Visibility::default(),
        CuadriculaFondo,
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

// Reposiciona la cuadrícula en saltos exactos de 50px para que sea infinita al mover la cámara
pub fn actualizar_cuadricula_infinita(
    q_cam: Query<&Transform, (With<CamaraPrincipal>, Without<CuadriculaFondo>)>,
    mut q_grid: Query<&mut Transform, (With<CuadriculaFondo>, Without<CamaraPrincipal>)>,
) {
    let Ok(cam_trf) = q_cam.single() else { return; };
    let paso = 50.0;
    let snap_x = (cam_trf.translation.x / paso).round() * paso;
    let snap_y = (cam_trf.translation.y / paso).round() * paso;

    for mut grid_trf in q_grid.iter_mut() {
        grid_trf.translation.x = snap_x;
        grid_trf.translation.y = snap_y;
    }
}

// --- Bloque 3: Dibujar Línea Temporal con Codos ---
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

            gizmos.line_2d(
                punto_anterior, 
                pos_mundo, 
                config.col_cable_tmp
            );
        }
    }
}

// --- Bloque 4: Dibujar Cables Permanentes con Puentes de Salto y Soldaduras ---
struct SegmentoCable {
    a: Vec2,
    b: Vec2,
    origen: Entity,
    color: Color,
}

// Calcula el punto donde se cortan dos segmentos si se cruzan realmente
fn interseccion_segmentos(
    a1: Vec2, 
    a2: Vec2, 
    b1: Vec2, 
    b2: Vec2
) -> Option<Vec2> {
    let r = a2 - a1;
    let s = b2 - b1;
    let rxs = r.perp_dot(s);

    if rxs.abs() < 0.001 {
        return None;
    }

    let qp = b1 - a1;
    let t = qp.perp_dot(s) / rxs;
    let u = qp.perp_dot(r) / rxs;

    // Evitamos contar los extremos como cruces
    if t > 0.06 && t < 0.94 && u > 0.06 && u < 0.94 {
        Some(a1 + r * t)
    } else {
        None
    }
}

pub fn dibuja_cables_permanentes(
    q_cables: Query<&Cable>,
    q_global_trf: Query<&GlobalTransform>,
    q_valores: Query<&ValorSeñal>,
    mut gizmos: Gizmos,
    config: Res<ConfigApp>,
) {
    let mut todos_segmentos: Vec<SegmentoCable> = Vec::new();
    let mut todos_codos: Vec<(Vec2, Entity, Color)> = Vec::new();

    for cable in q_cables.iter() {
        if let (Ok(trf_ori), Ok(trf_des)) = (
            q_global_trf.get(cable.origen), 
            q_global_trf.get(cable.destino)
        ) {
            let pos_origen = trf_ori.translation().truncate();
            let pos_destino = trf_des.translation().truncate();
            
            let mut color_cable = config.col_estado_0;
            if let Ok(val) = q_valores.get(cable.origen) {
                if val.0 {
                    color_cable = config.col_estado_1;
                }
            }
            
            let mut punto_anterior = pos_origen;
            for &codo in cable.puntos_intermedios.iter() {
                todos_segmentos.push(SegmentoCable {
                    a: punto_anterior,
                    b: codo,
                    origen: cable.origen,
                    color: color_cable,
                });
                todos_codos.push((codo, cable.origen, color_cable));
                punto_anterior = codo;
            }

            todos_segmentos.push(SegmentoCable {
                a: punto_anterior,
                b: pos_destino,
                origen: cable.origen,
                color: color_cable,
            });
        }
    }

    // 1. Dibujamos cada segmento aplicando puentes de salto si cruza un segmento anterior de otro origen
    let radio_salto = 7.0;
    for i in 0..todos_segmentos.len() {
        let seg = &todos_segmentos[i];
        let dir = seg.b - seg.a;
        let lon = dir.length();
        if lon < 0.1 {
            continue;
        }
        let u_dir = dir / lon;
        let normal = Vec2::new(-u_dir.y, u_dir.x);

        // Buscamos cruces con segmentos anteriores (j < i) que tengan distinto conector de origen
        let mut cruces_dist: Vec<f32> = Vec::new();
        for j in 0..i {
            let otro = &todos_segmentos[j];
            if otro.origen != seg.origen {
                if let Some(pto) = interseccion_segmentos(
                    seg.a, 
                    seg.b, 
                    otro.a, 
                    otro.b
                ) {
                    let d = seg.a.distance(pto);
                    if d > radio_salto && d < (lon - radio_salto) {
                        cruces_dist.push(d);
                    }
                }
            }
        }

        cruces_dist.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Trazamos el segmento deteniéndonos en cada cruce para dibujar un arco (puente)
        let mut cursor_d = 0.0;
        for d_cruce in cruces_dist {
            if d_cruce - radio_salto > cursor_d {
                let ini_tramo = seg.a + u_dir * cursor_d;
                let fin_tramo = seg.a + u_dir * (d_cruce - radio_salto);
                gizmos.line_2d(ini_tramo, fin_tramo, seg.color);

                // Dibujamos el puente semicircular de 8 lados sobre el cruce
                let centro_arco = seg.a + u_dir * d_cruce;
                let pasos_arco = 8;
                let mut p_ant_arco = fin_tramo;
                for k in 1..=pasos_arco {
                    let t = k as f32 / pasos_arco as f32;
                    let ang = std::f32::consts::PI * (1.0 - t);
                    let p_arco = centro_arco 
                        + u_dir * (ang.cos() * radio_salto) 
                        + normal * (ang.sin() * radio_salto);
                    gizmos.line_2d(p_ant_arco, p_arco, seg.color);
                    p_ant_arco = p_arco;
                }

                cursor_d = d_cruce + radio_salto;
            }
        }

        if cursor_d < lon {
            gizmos.line_2d(
                seg.a + u_dir * cursor_d, 
                seg.b, 
                seg.color
            );
        }
    }

    // 2. Dibujamos los codos y destacamos con un punto de soldadura grande las uniones eléctricas
    for (idx, &(pos_codo, orig_codo, col_codo)) in todos_codos.iter().enumerate() {
        let es_bifurcacion = todos_codos
            .iter()
            .enumerate()
            .any(|(j, &(p2, o2, _))| {
                j != idx && o2 == orig_codo && pos_codo.distance(p2) < 2.0
            });

        if es_bifurcacion {
            // Punto de soldadura relleno (dos círculos concéntricos)
            gizmos.circle_2d(pos_codo, 5.0, col_codo);
            gizmos.circle_2d(pos_codo, 2.5, col_codo);
        } else {
            gizmos.circle_2d(pos_codo, 3.0, col_codo);
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