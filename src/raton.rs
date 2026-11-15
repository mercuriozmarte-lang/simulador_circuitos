// ==========================================
// Fichero: src/raton.rs
// Bloque: Bloque 1: Importaciones
// ==========================================
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::componentes::*;
use crate::configuracion::ConfigApp;
use crate::menus::abrir_menu_contextual;
use crate::lineas::ajustar_ortogonal_90;
use crate::archivo::PeticionEntrarChip;
use crate::{
    crear_puerta,
    crear_interruptor,
    crear_led,
    crear_reloj,
    crear_display,
    crear_chip_personalizado,
};

// ==========================================
// Fichero: src/raton.rs
// Bloque: Bloque 2: Detectar Clic Izquierdo y Tecla D (Duplicar Pieza)
// ==========================================
pub fn detectar_clic_nodo(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    q_ui_abierta: Query<
        Entity, 
        Or<(
            With<MenuContextual>, 
            With<DialogoArchivo>, 
            With<DialogoEtiquetaPin>,
            With<SelectorCarga>
        )>
    >,
    q_nodos: Query<
        (Entity, &Transform, &NodoLogico, Option<&InstanciaChip>), 
        With<NodoLogico>
    >,
    q_conectores: Query<
        (Entity, &GlobalTransform, &TipoConector)
    >,
    q_cables: Query<(Entity, &Cable)>,
    mut controles_nodos: (
        Query<
            (&GlobalTransform, &mut BotonInterruptor, &mut Sprite),
            Without<ControlReloj>,
        >,
        Query<
            (&GlobalTransform, &mut ControlReloj, &mut Sprite, &Children),
            Without<BotonInterruptor>,
        >,
        Query<(&GlobalTransform, &BotonVerInteriorChip)>,
    ),
    mut q_textos_reloj: Query<&mut Text2d, With<TextoReloj>>,
    estado_app: Res<State<ModoApp>>,
    mut estado_linea: ResMut<EstadoCrearLinea>,
    mut codo_arrastrado: ResMut<CodoArrastrado>,
    mut next_state: ResMut<NextState<ModoApp>>,
    mut recursos_render: (
         Commands,
        ResMut<Assets<Mesh>>,
        ResMut<Assets<ColorMaterial>>,
    ),
    config: Res<ConfigApp>,
) {
    if !q_ui_abierta.is_empty() {
        return;
    }

    let Ok(window) = q_windows.single() else { return; };
    let Ok((cam, cam_trf)) = q_camera.single() else { return; };
    let Some(pos_pantalla) = window.cursor_position() else {
        return;
    };
    let Ok(pos_mundo) = cam.viewport_to_world_2d(
        cam_trf, 
        pos_pantalla
    ) else {
        return;
    };

    let (
        ref mut commands, 
        ref mut meshes, 
        ref mut materials
    ) = recursos_render;

    // ATAJO TECLA [ D ]: Duplicar cualquier Puerta, I/O o Chip bajo el cursor
    if keys.just_pressed(KeyCode::KeyD) {
        for (_, trf, nodo, opt_chip) in q_nodos.iter() {
            let centro = trf.translation.truncate();
            if pos_mundo.distance(centro) < config.rad_clic_nodo {
                let nueva_pos = Vec2::new(centro.x, centro.y - 100.0);
                match nodo.tipo {
                    TipoPuerta::And
                    | TipoPuerta::Or
                    | TipoPuerta::Not
                    | TipoPuerta::Nand
                    | TipoPuerta::Nor
                    | TipoPuerta::Xor
                    | TipoPuerta::Xnor => {
                        crear_puerta(
                            commands,
                            meshes,
                            materials,
                            nueva_pos,
                            nodo.tipo,
                            &config,
                        );
                    }
                    TipoPuerta::Interruptor => {
                        crear_interruptor(
                            commands,
                            meshes,
                            materials,
                            nueva_pos,
                            false,
                            "",
                            &config,
                        );
                    }
                    TipoPuerta::Led => {
                        crear_led(
                            commands,
                            meshes,
                            materials,
                            nueva_pos,
                            "",
                            &config,
                        );
                    }
                    TipoPuerta::Reloj => {
                        crear_reloj(
                            commands,
                            meshes,
                            materials,
                            nueva_pos,
                            1,
                            &config,
                        );
                    }
                    TipoPuerta::Display4Bits => {
                        crear_display(commands, nueva_pos, true, &config);
                    }
                    TipoPuerta::Display7Seg => {
                        crear_display(commands, nueva_pos, false, &config);
                    }
                    TipoPuerta::Chip => {
                        if let Some(chip) = opt_chip {
                            crear_chip_personalizado(
                                commands,
                                nueva_pos,
                                chip.datos.clone(),
                                &config,
                            );
                        }
                    }
                }
                return;
            }
        }
    }

    if keys.pressed(KeyCode::Space) || !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    if pos_pantalla.y >= (window.height() - 56.0) {
        return;
    }

    let (
        mut q_botones, 
        mut q_relojes, 
        q_botones_ver_chip
    ) = controles_nodos;

    // MODO CREANDO LÍNEA: Añadir codo (con soporte 90° en tecla L) o finalizar cable
    if *estado_app.get() == ModoApp::CreandoLinea {
        for (ent_destino, g_trf, tipo) in q_conectores.iter() {
            let pos_conector = g_trf.translation().truncate();
            if pos_mundo.distance(pos_conector) < config.rad_clic_con {
                if *tipo == TipoConector::Entrada {
                    let ocupada = q_cables
                        .iter()
                        .any(|(_, c)| c.destino == ent_destino);

                    if !ocupada {
                        if let Some(orig) = estado_linea.origen_entidad {
                            commands.spawn(Cable {
                                origen: orig,
                                destino: ent_destino,
                                puntos_intermedios: estado_linea
                                    .puntos_intermedios
                                    .clone(),
                            });
                        }
                    }
                    estado_linea.puntos_intermedios.clear();
                    next_state.set(ModoApp::Edicion);
                    return;
                }
            }
        }

        let pto_previo = estado_linea
            .puntos_intermedios
            .last()
            .copied()
            .unwrap_or(estado_linea.origen_pos);

        let pos_objetivo = if keys.pressed(KeyCode::KeyL) {
            ajustar_ortogonal_90(pto_previo, pos_mundo)
        } else {
            pos_mundo
        };

        let paso_codo = config.paso_snap_codo;
        let codo_snap = Vec2::new(
            (pos_objetivo.x / paso_codo).round() * paso_codo,
            (pos_objetivo.y / paso_codo).round() * paso_codo,
        );
        estado_linea.puntos_intermedios.push(codo_snap);
        return;
    }

    // MODO EDICIÓN NORMAL:
    // 1. Prioridad 1: Clic en conector de Salida para empezar un cable
    for (ent_conector, g_trf, tipo) in q_conectores.iter() {
        let pos_conector = g_trf.translation().truncate();
        if pos_mundo.distance(pos_conector) < config.rad_clic_con {
            if *tipo == TipoConector::Salida {
                estado_linea.origen_entidad = Some(ent_conector);
                estado_linea.origen_pos = pos_conector;
                estado_linea.puntos_intermedios.clear();
                next_state.set(ModoApp::CreandoLinea);
                return; 
            }
        }
    }

    // 1B. Clic sobre una esquina (codo) de un cable ya creado para moverla
    for (ent_cable, cable) in q_cables.iter() {
        for (idx, &pos_codo) in cable.puntos_intermedios.iter().enumerate() {
            if pos_mundo.distance(pos_codo) < 10.0 {
                codo_arrastrado.seleccion = Some((ent_cable, idx));
                return;
            }
        }
    }

    // 2. Prioridad 2: Clic en el cuadrado de cambio de estado (0 / 1)
    for (g_trf, mut boton, mut sprite) in q_botones.iter_mut() {
        let pos_boton = g_trf.translation().truncate();
        if (pos_mundo.x - pos_boton.x).abs() < 13.0 
            && (pos_mundo.y - pos_boton.y).abs() < 13.0 
        {
            boton.estado = !boton.estado;
            if boton.estado {
                sprite.color = config.col_estado_1;
            } else {
                sprite.color = config.col_estado_0;
            }
            return;
        }
    }

    // 2B. Prioridad 2B: Clic en el cuadrado de frecuencia del Reloj
    let etiquetas_frec = ["OFF", "1Hz", "2Hz", "5Hz", "10Hz"];
    for (g_trf, mut reloj, mut sprite, hijos) in q_relojes.iter_mut() {
        let pos_ctrl = g_trf.translation().truncate();
        if (pos_mundo.x - pos_ctrl.x).abs() < 17.0 
            && (pos_mundo.y - pos_ctrl.y).abs() < 13.0 
        {
            reloj.indice_frec = 
                (reloj.indice_frec + 1) % etiquetas_frec.len();
            reloj.acumulador = 0.0;

            if reloj.indice_frec == 0 {
                reloj.estado = false;
                sprite.color = config.col_estado_0;
            }

            for hijo in hijos.iter() {
                if let Ok(mut txt) = q_textos_reloj.get_mut(hijo) {
                    txt.0 = 
                        etiquetas_frec[reloj.indice_frec].to_string();
                }
            }
            return;
        }
    }

    // 2C. Prioridad 2C: Clic en el pequeño botón [VER] de un Chip
    for (g_trf, btn_ver) in q_botones_ver_chip.iter() {
        let pos_btn = g_trf.translation().truncate();
        if (pos_mundo.x - pos_btn.x).abs() < 21.0 
            && (pos_mundo.y - pos_btn.y).abs() < 9.0 
        {
            commands.spawn(PeticionEntrarChip(btn_ver.archivo_chp.clone()));
            return;
        }
    }

    // 3. Prioridad 3: Clic en el cuerpo del nodo para arrastrarlo
    for (entidad, trf, _, _) in q_nodos.iter() {
        let centro_nodo = trf.translation.truncate();
        if pos_mundo.distance(centro_nodo) < config.rad_clic_nodo {
            commands.entity(entidad).insert(Arrastrando);
            return; 
        }
    }
}

// --- Bloque 3: Soltar Ratón, Snap Diferenciado (Puertas vs I/O) e Imantar a Barras ---
pub fn soltar_nodo(
    buttons: Res<ButtonInput<MouseButton>>,
    mut q_arrastrando: Query<
        (Entity, &mut Transform, &NodoLogico), 
        With<Arrastrando>
    >,
    q_barras: Query<(&Transform, &BarraChip), Without<Arrastrando>>,
    q_conectores: Query<
        (Entity, &GlobalTransform, &TipoConector)
    >,
    mut q_cables: Query<&mut Cable>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    estado_app: Res<State<ModoApp>>,
    mut estado_linea: ResMut<EstadoCrearLinea>,
    mut codo_arrastrado: ResMut<CodoArrastrado>,
    mut next_state: ResMut<NextState<ModoApp>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if !buttons.just_released(MouseButton::Left) {
        return;
    }

    // 0. Si estábamos arrastrando el codo de un cable, lo encajamos a config.paso_snap_codo (5px)
    if let Some((ent_cable, idx)) = codo_arrastrado.seleccion.take() {
        if let Ok(mut cable) = q_cables.get_mut(ent_cable) {
            if let Some(codo) = cable.puntos_intermedios.get_mut(idx) {
                let paso_codo = config.paso_snap_codo;
                codo.x = (codo.x / paso_codo).round() * paso_codo;
                codo.y = (codo.y / paso_codo).round() * paso_codo;
            }
        }
        return;
    }

    // 1. Soltar nodos arrastrados: Interruptores y LEDs usan paso_snap_io (5px)
    //    para poder colocarse en posiciones intermedias; el resto usa paso_snap_puerta (50px).
    for (entidad, mut trf, nodo) in q_arrastrando.iter_mut() {
        let paso = if matches!(
            nodo.tipo, 
            TipoPuerta::Interruptor | TipoPuerta::Led
        ) {
            config.paso_snap_io
        } else {
            config.paso_snap_puerta
        };

        let mut x_final = (trf.translation.x / paso).round() * paso;
        let y_final = (trf.translation.y / paso).round() * paso;

        let mut barra_pegada = None;

        for (trf_barra, tipo_barra) in q_barras.iter() {
            let x_barra = trf_barra.translation.x;
            let dist_x = (trf.translation.x - x_barra).abs();

            if *tipo_barra == BarraChip::IzquierdaEntradas 
                && nodo.tipo == TipoPuerta::Interruptor 
                && dist_x < 60.0 
            {
                x_final = x_barra;
                barra_pegada = Some(BarraChip::IzquierdaEntradas);
            }

            if *tipo_barra == BarraChip::DerechaSalidas 
                && nodo.tipo == TipoPuerta::Led 
                && dist_x < 60.0 
            {
                x_final = x_barra;
                barra_pegada = Some(BarraChip::DerechaSalidas);
            }
        }

        trf.translation.x = x_final;
        trf.translation.y = y_final;

        if let Some(b) = barra_pegada {
            commands.entity(entidad).insert(PegadoABarra(b));
        } else {
            commands.entity(entidad).remove::<PegadoABarra>();
        }

        commands.entity(entidad).remove::<Arrastrando>();
    }

    // 2. Cerrar cable directo sin codos al soltar sobre una Entrada
    if *estado_app.get() == ModoApp::CreandoLinea 
        && estado_linea.puntos_intermedios.is_empty() 
    {
        let Ok(window) = q_windows.single() else { return; };
        let Ok((cam, cam_trf)) = q_camera.single() else { return; };

        if let Some(pos_pantalla) = window.cursor_position() {
            if let Ok(pos_mundo) = cam.viewport_to_world_2d(
                cam_trf, 
                pos_pantalla
            ) {
                for (ent_destino, g_trf, tipo) in q_conectores.iter() {
                    let pos_conector = g_trf.translation().truncate();
                    if pos_mundo.distance(pos_conector) < config.rad_clic_con {
                        if *tipo == TipoConector::Entrada {
                            let entrada_ocupada = q_cables
                                .iter()
                                .any(|c| c.destino == ent_destino);

                            if !entrada_ocupada {
                                if let Some(orig) = 
                                    estado_linea.origen_entidad 
                                {
                                    commands.spawn(Cable {
                                        origen: orig,
                                        destino: ent_destino,
                                        puntos_intermedios: Vec::new(),
                                    });
                                }
                            }
                            estado_linea.puntos_intermedios.clear();
                            next_state.set(ModoApp::Edicion);
                            return;
                        }
                    }
                }
            }
        }
    }
}

// --- Bloque 4: Mover Nodo o Codo Arrastrado (con Soporte 90° en Tecla L) ---
pub fn mover_nodo_arrastrado(
    keys: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    q_global_trf: Query<&GlobalTransform>,
    mut q_arrastrando: Query<&mut Transform, With<Arrastrando>>,
    codo_arrastrado: Res<CodoArrastrado>,
    mut q_cables: Query<&mut Cable>,
) {
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

            if let Some((ent_cable, idx)) = codo_arrastrado.seleccion {
                if let Ok(mut cable) = q_cables.get_mut(ent_cable) {
                    // Punto anterior al codo para bloquear a 90° si pulsa L
                    let pto_ref = if idx > 0 {
                        cable.puntos_intermedios.get(idx - 1).copied()
                    } else {
                        q_global_trf
                            .get(cable.origen)
                            .ok()
                            .map(|t| t.translation().truncate())
                    };

                    if let Some(codo) = cable.puntos_intermedios.get_mut(idx) {
                        if keys.pressed(KeyCode::KeyL) {
                            if let Some(base) = pto_ref {
                                *codo = ajustar_ortogonal_90(base, pos_mundo);
                            } else {
                                *codo = pos_mundo;
                            }
                        } else {
                            *codo = pos_mundo;
                        }
                    }
                }
            }
        }
    }
}

// --- Bloque 5: Borrar Nodo, Codo, Cable o Abrir Menú con Clic Derecho ---
pub fn borrar_con_clic_derecho(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    q_nodos: Query<(Entity, &Transform), With<NodoLogico>>,
    mut q_cables: Query<(Entity, &mut Cable)>,
    q_global_trf: Query<&GlobalTransform>,
    q_menu: Query<Entity, With<MenuContextual>>,
    q_submenu: Query<Entity, With<SubMenuContextual>>,
    estado_app: Res<State<ModoApp>>,
    mut estado_linea: ResMut<EstadoCrearLinea>,
    mut next_state: ResMut<NextState<ModoApp>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }

    if *estado_app.get() == ModoApp::CreandoLinea {
        if estado_linea.puntos_intermedios.pop().is_none() {
            next_state.set(ModoApp::Edicion);
        }
        return;
    }

    for ent_menu in q_menu.iter() {
        commands.entity(ent_menu).despawn();
    }
    for ent_sub in q_submenu.iter() {
        commands.entity(ent_sub).despawn();
    }

    let Ok(window) = q_windows.single() else { return; };
    let Ok((cam, cam_trf)) = q_camera.single() else { return; };

    if let Some(pos_pantalla) = window.cursor_position() {
        if let Ok(pos_mundo) = cam.viewport_to_world_2d(
            cam_trf, 
            pos_pantalla
        ) {
            for (ent_nodo, trf) in q_nodos.iter() {
                let centro = trf.translation.truncate();
                if pos_mundo.distance(centro) < config.rad_clic_nodo {
                    commands.entity(ent_nodo).despawn();
                    return;
                }
            }

            let mut codo_a_borrar = None;
            for (ent_c, cable) in q_cables.iter() {
                if let Some(idx) = cable
                    .puntos_intermedios
                    .iter()
                    .position(|&c| pos_mundo.distance(c) < 10.0)
                {
                    codo_a_borrar = Some((ent_c, idx));
                    break;
                }
            }

            if let Some((ent_c, idx)) = codo_a_borrar {
                if let Ok((_, mut cable)) = q_cables.get_mut(ent_c) {
                    cable.puntos_intermedios.remove(idx);
                }
                return;
            }

            for (ent_cable, cable) in q_cables.iter() {
                if let (Ok(trf_ori), Ok(trf_des)) = (
                    q_global_trf.get(cable.origen),
                    q_global_trf.get(cable.destino),
                ) {
                    let mut puntos = Vec::new();
                    puntos.push(trf_ori.translation().truncate());
                    for &codo in cable.puntos_intermedios.iter() {
                        puntos.push(codo);
                    }
                    puntos.push(trf_des.translation().truncate());

                    for ventana_pts in puntos.windows(2) {
                        let a = ventana_pts[0];
                        let b = ventana_pts[1];
                        let ab = b - a;
                        let len_sq = ab.length_squared();
                        if len_sq > 0.001 {
                            let ap = pos_mundo - a;
                            let t = (ap.dot(ab) / len_sq).clamp(0.0, 1.0);
                            let cercano = a + ab * t;
                            if pos_mundo.distance(cercano) < 8.0 {
                                commands.entity(ent_cable).despawn();
                                return;
                            }
                        }
                    }
                }
            }

            abrir_menu_contextual(&mut commands, pos_mundo);
        }
    }
}