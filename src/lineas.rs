// ==========================================
// Fichero: src/lineas.rs
// ==========================================

// --- Bloque 1: Importaciones, Ajuste Ortogonal y Dibujo de Cinta Multicolor de Bus ---
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::componentes::*;
use crate::configuracion::ConfigApp;

pub fn ajustar_ortogonal_90(
    origen: Vec2, 
    destino: Vec2
) -> Vec2 {
    let dx = (destino.x - origen.x).abs();
    let dy = (destino.y - origen.y).abs();
    if dx >= dy {
        Vec2::new(destino.x, origen.y)
    } else {
        Vec2::new(origen.x, destino.y)
    }
}

// Dibuja un cable de Bus de N hilos con envolvente negra y N líneas paralelas verde/rojo
fn dibujar_cinta_bus(
    gizmos: &mut Gizmos,
    puntos: &[Vec2],
    estados: &[bool],
    num_hilos: usize,
    config: &ConfigApp,
) {
    let n = num_hilos.max(2);
    let paso_hilo = 2.0f32;
    let semiancho = ((n - 1) as f32 * paso_hilo) / 2.0;
    let col_borde_bus = Color::srgb(0.08, 0.10, 0.14);

    for ventana in puntos.windows(2) {
        let a = ventana[0];
        let b = ventana[1];
        let dir = (b - a).normalize_or_zero();
        if dir == Vec2::ZERO {
            continue;
        }
        let perp = Vec2::new(-dir.y, dir.x);

        // 1. Envolvente negra exterior a ambos lados de la cinta del Bus
        let off_ext = perp * (semiancho + 2.2);
        gizmos.line_2d(a + off_ext, b + off_ext, col_borde_bus);
        gizmos.line_2d(a - off_ext, b - off_ext, col_borde_bus);

        // 2. Las N pistas paralelas interiores coloreadas en verde (1) o rojo (0)
        for i in 0..n {
            let desp = semiancho - (i as f32 * paso_hilo);
            let off_i = perp * desp;
            let bit_activo = estados.get(i).copied().unwrap_or(false);
            let col_hilo = if bit_activo {
                config.col_estado_1
            } else {
                config.col_estado_0
            };
            gizmos.line_2d(a + off_i, b + off_i, col_hilo);
        }
    }

    // 3. Anillos circulares en los codos de unión del Bus (como en tu boceto)
    let radio_codo = (semiancho + 3.5).max(6.5);
    for &codo in puntos.iter().skip(1).take(puntos.len().saturating_sub(2)) {
        gizmos.circle_2d(codo, radio_codo, col_borde_bus);
        gizmos.circle_2d(codo, radio_codo * 0.55, Color::WHITE);
    }
}

// --- Bloque 2: Creación y Seguimiento de la Cuadrícula Infinita de Fondo ---
pub fn crear_cuadricula_fondo(
    commands: &mut Commands,
    config_store: &mut ResMut<GizmoConfigStore>,
    config: &ConfigApp,
) {
    let (gizmo_config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    gizmo_config.line.width = config.grosor_cable;

    let paso = config.paso_snap_puerta;
    let num_lineas: i32 = 80;
    let longitud = (num_lineas as f32) * paso * 2.0;

    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::default(),
        CuadriculaFondo,
    ))
    .with_children(|padre| {
        for i in -num_lineas..=num_lineas {
            let coord = (i as f32) * paso;

            padre.spawn((
                Sprite {
                    color: config.col_cuadricula,
                    custom_size: Some(Vec2::new(1.0, longitud)),
                    ..default()
                },
                Transform::from_xyz(coord, 0.0, 0.0),
            ));

            padre.spawn((
                Sprite {
                    color: config.col_cuadricula,
                    custom_size: Some(Vec2::new(longitud, 1.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, coord, 0.0),
            ));
        }
    });
}

pub fn actualizar_cuadricula_infinita(
    q_camara: Query<&Transform, (With<CamaraPrincipal>, Without<CuadriculaFondo>)>,
    mut q_cuadricula: Query<&mut Transform, With<CuadriculaFondo>>,
    config: Res<ConfigApp>,
) {
    let Ok(trf_cam) = q_camara.single() else { return; };
    let paso = config.paso_snap_puerta;

    let snap_x = (trf_cam.translation.x / paso).round() * paso;
    let snap_y = (trf_cam.translation.y / paso).round() * paso;

    for mut trf_grid in q_cuadricula.iter_mut() {
        trf_grid.translation.x = snap_x;
        trf_grid.translation.y = snap_y;
    }
}

// --- Bloque 3: Renderizado de la Línea o Bus Temporal (Mientras se traza) ---
pub fn dibuja_linea_temporal(
    keys: Res<ButtonInput<KeyCode>>,
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

    let Ok(window) = q_windows.single() else { 
        return; 
    };
    let Ok((cam, cam_trf)) = q_camera.single() else { 
        return; 
    };

    if let Some(pos_pantalla) = window.cursor_position() {
        if let Ok(pos_mundo) = cam.viewport_to_world_2d(
            cam_trf, 
            pos_pantalla
        ) {
            let mut puntos = Vec::with_capacity(
                estado_linea.puntos_intermedios.len() + 2
            );
            puntos.push(estado_linea.origen_pos);

            for &codo in estado_linea.puntos_intermedios.iter() {
                puntos.push(codo);
            }

            let ultimo_pto = *puntos.last().unwrap();
            let pto_cursor = if keys.pressed(KeyCode::KeyL) {
                ajustar_ortogonal_90(ultimo_pto, pos_mundo)
            } else {
                pos_mundo
            };
            puntos.push(pto_cursor);

            if estado_linea.es_bus {
                let n = estado_linea.num_hilos_bus.max(2);
                let estados_tmp = vec![false; n];
                dibujar_cinta_bus(
                    &mut gizmos, 
                    &puntos, 
                    &estados_tmp, 
                    n, 
                    &config
                );
            } else {
                for ventana in puntos.windows(2) {
                    gizmos.line_2d(
                        ventana[0], 
                        ventana[1], 
                        config.col_cable_tmp
                    );
                }

                for &codo in estado_linea.puntos_intermedios.iter() {
                    gizmos.circle_2d(
                        codo, 
                        4.5, 
                        config.col_cable_tmp
                    );
                }
            }
        }
    }
}

// ==========================================
// Fichero: src/lineas.rs
// Bloque: Bloque 4: Renderizado de Cables Permanentes, Buses Multicolor (Recortados bajo el Menú) y Selección
// ==========================================
fn punto_dentro_de_menus(
    p: Vec2,
    rects_menu: &[(Vec2, Vec2)],
) -> bool {
    for &(centro, semitam) in rects_menu.iter() {
        if (p.x - centro.x).abs() <= semitam.x 
            && (p.y - centro.y).abs() <= semitam.y 
        {
            return true;
        }
    }
    false
}

fn trazar_segmento_sin_tapar_menu(
    gizmos: &mut Gizmos,
    a: Vec2,
    b: Vec2,
    color: Color,
    rects_menu: &[(Vec2, Vec2)],
) {
    if rects_menu.is_empty() {
        gizmos.line_2d(a, b, color);
        return;
    }

    // Subdividimos en pequeños pasos solo cuando hay un menú abierto para recortar lo que queda debajo
    let pasos = 24;
    for s in 0..pasos {
        let t0 = (s as f32) / (pasos as f32);
        let t1 = ((s + 1) as f32) / (pasos as f32);
        let p0 = a.lerp(b, t0);
        let p1 = a.lerp(b, t1);
        let medio = (p0 + p1) * 0.5;
        if !punto_dentro_de_menus(medio, rects_menu) {
            gizmos.line_2d(p0, p1, color);
        }
    }
}

pub fn dibuja_cables_permanentes(
    q_cables: Query<&Cable>,
    q_cables_bus: Query<&CableBus>,
    q_puertos_bus: Query<(&GlobalTransform, &PuertoTroncoBus)>,
    q_conectores: Query<(&GlobalTransform, &ValorSeñal)>,
    q_seleccionados: Query<&Transform, With<Seleccionado>>,
    q_menus_activos: Query<(&Transform, &Sprite), Or<(With<MenuContextual>, With<SubMenuContextual>)>>,
    caja_sel: Res<CajaSeleccion>,
    mut gizmos: Gizmos,
    config: Res<ConfigApp>,
) {
    let mut rects_menu: Vec<(Vec2, Vec2)> = Vec::new();
    for (trf_m, spr_m) in q_menus_activos.iter() {
        let tam = spr_m.custom_size.unwrap_or(Vec2::new(150.0, 260.0));
        rects_menu.push((trf_m.translation.truncate(), tam * 0.5 + Vec2::splat(4.0)));
    }

    // 1. Dibujar cables normales y sus codos (sin tapar el menú contextual)
    for cable in q_cables.iter() {
        let Ok((trf_ori, val_ori)) = q_conectores.get(cable.origen) else {
            continue;
        };
        let Ok((trf_des, _)) = q_conectores.get(cable.destino) else {
            continue;
        };

        let color = if val_ori.0 {
            config.col_estado_1
        } else {
            config.col_estado_0
        };

        let mut puntos = Vec::with_capacity(
            cable.puntos_intermedios.len() + 2
        );
        puntos.push(trf_ori.translation().truncate());
        for &codo in cable.puntos_intermedios.iter() {
            puntos.push(codo);
        }
        puntos.push(trf_des.translation().truncate());

        for ventana in puntos.windows(2) {
            trazar_segmento_sin_tapar_menu(
                &mut gizmos,
                ventana[0],
                ventana[1],
                color,
                &rects_menu,
            );
        }

        for &codo in cable.puntos_intermedios.iter() {
            if !punto_dentro_de_menus(codo, &rects_menu) {
                gizmos.circle_2d(codo, 4.5, color);
            }
        }
    }

    // 2. Dibujar todos los Buses (CableBus) con sus N pistas verdes/rojas (sin tapar el menú)
    for cb in q_cables_bus.iter() {
        let mut pos_ori = None;
        let mut pos_des = None;
        for (g_trf, p) in q_puertos_bus.iter() {
            if p.nodo_padre == cb.origen_nodo {
                pos_ori = Some(g_trf.translation().truncate());
            }
            if p.nodo_padre == cb.destino_nodo {
                pos_des = Some(g_trf.translation().truncate());
            }
        }

        if let (Some(p_a), Some(p_b)) = (pos_ori, pos_des) {
            let mut puntos = Vec::with_capacity(
                cb.puntos_intermedios.len() + 2
            );
            puntos.push(p_a);
            for &codo in cb.puntos_intermedios.iter() {
                puntos.push(codo);
            }
            puntos.push(p_b);

            if rects_menu.is_empty() {
                dibujar_cinta_bus(
                    &mut gizmos,
                    &puntos,
                    &cb.estados,
                    cb.num_hilos,
                    &config,
                );
            } else {
                let n = cb.num_hilos.max(2);
                let paso_hilo = 2.0f32;
                let semiancho = ((n - 1) as f32 * paso_hilo) / 2.0;
                let col_borde_bus = Color::srgb(0.08, 0.10, 0.14);

                for ventana in puntos.windows(2) {
                    let a = ventana[0];
                    let b = ventana[1];
                    let dir = (b - a).normalize_or_zero();
                    if dir == Vec2::ZERO {
                        continue;
                    }
                    let perp = Vec2::new(-dir.y, dir.x);
                    let off_ext = perp * (semiancho + 2.2);
                    trazar_segmento_sin_tapar_menu(
                        &mut gizmos,
                        a + off_ext,
                        b + off_ext,
                        col_borde_bus,
                        &rects_menu,
                    );
                    trazar_segmento_sin_tapar_menu(
                        &mut gizmos,
                        a - off_ext,
                        b - off_ext,
                        col_borde_bus,
                        &rects_menu,
                    );
                    for i in 0..n {
                        let desp = semiancho - (i as f32 * paso_hilo);
                        let off_i = perp * desp;
                        let bit_activo = cb.estados.get(i).copied().unwrap_or(false);
                        let col_hilo = if bit_activo {
                            config.col_estado_1
                        } else {
                            config.col_estado_0
                        };
                        trazar_segmento_sin_tapar_menu(
                            &mut gizmos,
                            a + off_i,
                            b + off_i,
                            col_hilo,
                            &rects_menu,
                        );
                    }
                }
            }
        }
    }

    // 3. Dibujar marco celeste alrededor de los nodos seleccionados
    let col_sel = Color::srgb(0.15, 0.65, 1.0);
    for trf_sel in q_seleccionados.iter() {
        let c = trf_sel.translation.truncate();
        if !punto_dentro_de_menus(c, &rects_menu) {
            gizmos.rect_2d(
                c, 
                Vec2::new(92.0, 56.0), 
                col_sel
            );
        }
    }

    if caja_sel.activa {
        let centro = (caja_sel.inicio + caja_sel.actual) / 2.0;
        let tam = (caja_sel.actual - caja_sel.inicio).abs();
        gizmos.rect_2d(centro, tam, col_sel);
    }
}