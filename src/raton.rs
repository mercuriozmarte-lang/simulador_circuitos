// ==========================================
// Fichero: src/raton.rs
// ==========================================

// --- Bloque 1: Importaciones ---
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
    crear_conector_bus,
};

// ==========================================
// Fichero: src/raton.rs
// Bloque: Bloque 2: Detectar Clic Izquierdo, Trazado de Cables y Buses, Conector de Bus y Selección
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
    // Agrupamos las consultas de lectura del circuito en una tupla para no superar los 16 parámetros de Bevy
    consultas_circuito: (
        Query<
            (
                Entity, 
                &Transform, 
                &NodoLogico, 
                Option<&InstanciaChip>,
                Option<&DatosConectorBus>,
                Option<&Arrastrando>,
                Option<&Seleccionado>,
            ), 
            With<NodoLogico>
        >,
        Query<(
            Entity, 
            &GlobalTransform, 
            &TipoConector,
            Option<&ConectorBidireccional>,
        )>,
        Query<(&GlobalTransform, &PuertoTroncoBus)>,
        Query<(Entity, &Cable)>,
        Query<(Entity, &CableBus)>,
    ),
    controles_nodos: (
        Query<
            (&GlobalTransform, &mut BotonInterruptor, &mut Sprite),
            Without<ControlReloj>,
        >,
        Query<
            (&GlobalTransform, &mut ControlReloj, &mut Sprite, &Children),
            Without<BotonInterruptor>,
        >,
        Query<(&GlobalTransform, &BotonVerInteriorChip)>,
        Query<(&GlobalTransform, &ControlFlechaBus)>,
        Query<(&GlobalTransform, &BotonReguladorBus)>,
        Query<(&GlobalTransform, &AsaMoverBarra)>,
        Query<(&GlobalTransform, &AsaAltoBarra)>,
    ),
    mut q_textos_reloj: Query<&mut Text2d, With<TextoReloj>>,
    estado_app: Res<State<ModoApp>>,
    mut estados_edicion: (
        ResMut<EstadoCrearLinea>,
        ResMut<CodoArrastrado>,
        ResMut<EstadoArrastreBarra>,
        ResMut<CajaSeleccion>,
        ResMut<NextState<ModoApp>>,
    ),
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
        ref q_nodos,
        ref q_conectores,
        ref q_puertos_bus,
        ref q_cables,
        ref q_cables_bus,
    ) = consultas_circuito;

    let (
        ref mut commands, 
        ref mut meshes, 
        ref mut materials
    ) = recursos_render;
    let (
        ref mut estado_linea,
        ref mut codo_arrastrado,
        ref mut arrastre_barra,
        ref mut caja_sel,
        ref mut next_state,
    ) = estados_edicion;

    // 0. BORRAR BLOQUE SELECCIONADO CON TECLA [DELETE / SUPR] O [BACKSPACE]
    if keys.just_pressed(KeyCode::Delete) 
        || keys.just_pressed(KeyCode::Backspace) 
    {
        let mut borrado = false;
        for (ent, _, _, _, _, _, opt_sel) in q_nodos.iter() {
            if opt_sel.is_some() {
                commands.entity(ent).despawn();
                borrado = true;
            }
        }
        if borrado {
            return;
        }
    }

    // 0A. ATAJOS SOBRE CONECTOR DE BUS (En mano o bajo el cursor)
    let cambio_r = keys.just_pressed(KeyCode::KeyR);
    let cambio_f = keys.just_pressed(KeyCode::KeyF);
    let subir_n = keys.just_pressed(KeyCode::ArrowUp)
        || keys.just_pressed(KeyCode::PageUp)
        || keys.just_pressed(KeyCode::NumpadAdd)
        || keys.just_pressed(KeyCode::Equal);
    let bajar_n = keys.just_pressed(KeyCode::ArrowDown)
        || keys.just_pressed(KeyCode::PageDown)
        || keys.just_pressed(KeyCode::NumpadSubtract)
        || keys.just_pressed(KeyCode::Minus);

    if cambio_r || cambio_f || subir_n || bajar_n {
        for (ent, trf, nodo, _, opt_bus, opt_arr, _) in q_nodos.iter() {
            if nodo.tipo != TipoPuerta::ConectorBus {
                continue;
            }
            let Some(&datos_b) = opt_bus else { continue; };
            let centro = trf.translation.truncate();
            let radio_bus = (datos_b.num_hilos as f32 * 14.0).max(65.0);
            let es_objetivo = opt_arr.is_some() 
                || pos_mundo.distance(centro) < radio_bus;

            if es_objetivo {
                let mut nuevos_datos = datos_b;
                if cambio_r {
                    nuevos_datos.rotacion = (nuevos_datos.rotacion + 1) % 4;
                }
                if cambio_f {
                    nuevos_datos.modo = nuevos_datos.modo.siguiente();
                }
                if subir_n {
                    nuevos_datos.num_hilos = 
                        (nuevos_datos.num_hilos + 1).min(16);
                }
                if bajar_n {
                    nuevos_datos.num_hilos = 
                        nuevos_datos.num_hilos.saturating_sub(1).max(2);
                }

                let offset_previo = opt_arr.map(|a| a.offset);
                commands.entity(ent).despawn();
                let (nuevo_ent, _, _) = crear_conector_bus(
                    commands,
                    centro,
                    nuevos_datos,
                    &config,
                );
                if let Some(off) = offset_previo {
                    commands
                        .entity(nuevo_ent)
                        .insert(Arrastrando { offset: off });
                }
                return;
            }
        }
    }

    // 0B. SI YA LLEVAMOS PIEZAS ENGANCHADAS AL CURSOR
    let mut en_mano: Vec<(Entity, TipoPuerta, Option<DatosChip>, f32)> = 
        Vec::new();
    for (ent, _, nodo, opt_chip, _, opt_arr, _) in q_nodos.iter() {
        if let Some(arr) = opt_arr {
            en_mano.push((
                ent,
                nodo.tipo,
                opt_chip.map(|c| c.datos.clone()),
                arr.offset.y,
            ));
        }
    }

    if !en_mano.is_empty() {
        if keys.just_pressed(KeyCode::KeyD) 
            || keys.just_pressed(KeyCode::ArrowUp) 
        {
            let (_, tipo_base, datos_base, min_offset) = en_mano
                .iter()
                .min_by(|a, b| a.3.partial_cmp(&b.3).unwrap())
                .cloned()
                .unwrap();

            let paso = config.paso_snap_puerta;
            let sep_vertical = if let Some(ref d) = datos_base {
                let max_p = d
                    .pines_entrada
                    .len()
                    .max(d.pines_salida.len())
                    .max(1);
                let alto_est = (
                    max_p as f32 * config.sep_pines_chip + 36.0
                ).max(100.0);
                (alto_est / paso).ceil() * paso
            } else {
                50.0
            };

            let nuevo_offset = min_offset - sep_vertical;
            let pos_nueva = Vec2::new(
                pos_mundo.x, 
                pos_mundo.y + nuevo_offset
            );

            if tipo_base == TipoPuerta::Chip {
                if let Some(datos) = datos_base {
                    let (ent_nuevo, _, _) = crear_chip_personalizado(
                        commands,
                        pos_nueva,
                        datos,
                        &config,
                    );
                    commands
                        .entity(ent_nuevo)
                        .insert(Arrastrando { 
                            offset: Vec2::new(0.0, nuevo_offset) 
                        });
                }
            }
            return;
        }

        if keys.just_pressed(KeyCode::ArrowDown) && en_mano.len() > 1 {
            if let Some((ent_ultimo, _, _, _)) = en_mano
                .iter()
                .min_by(|a, b| a.3.partial_cmp(&b.3).unwrap())
            {
                commands.entity(*ent_ultimo).despawn();
            }
            return;
        }

        if keys.just_pressed(KeyCode::Escape) {
            for (ent, _, _, _) in en_mano {
                commands.entity(ent).despawn();
            }
            return;
        }
    }

    // 0C. Duplicar pieza bajo el cursor con tecla [D]
    if en_mano.is_empty() && keys.just_pressed(KeyCode::KeyD) {
        for (_, trf, nodo, opt_chip, opt_bus, _, _) in q_nodos.iter() {
            let centro = trf.translation.truncate();
            if pos_mundo.distance(centro) < config.rad_clic_nodo {
                let nueva_pos = Vec2::new(
                    centro.x, 
                    centro.y - config.sep_pines_chip
                );
                match nodo.tipo {
                    TipoPuerta::And
                    | TipoPuerta::Or
                    | TipoPuerta::Not
                    | TipoPuerta::Nand
                    | TipoPuerta::Nor
                    | TipoPuerta::Xor
                    | TipoPuerta::Xnor
                    | TipoPuerta::TriState => {
                        crear_puerta(
                            commands,
                            meshes,
                            materials,
                            Vec2::new(centro.x, centro.y - 50.0),
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
                            Vec2::new(centro.x, centro.y - 100.0),
                            1,
                            &config,
                        );
                    }
                    TipoPuerta::Display4Bits => {
                        crear_display(
                            commands, 
                            Vec2::new(centro.x, centro.y - 100.0), 
                            true, 
                            &config
                        );
                    }
                    TipoPuerta::Display7Seg => {
                        crear_display(
                            commands, 
                            Vec2::new(centro.x, centro.y - 200.0), 
                            false, 
                            &config
                        );
                    }
                    TipoPuerta::ConectorBus => {
                        if let Some(&datos_b) = opt_bus {
                            let (ent_b, _, _) = crear_conector_bus(
                                commands,
                                pos_mundo,
                                datos_b,
                                &config,
                            );
                            commands
                                .entity(ent_b)
                                .insert(Arrastrando { offset: Vec2::ZERO });
                        }
                    }
                    TipoPuerta::Chip => {
                        if let Some(chip) = opt_chip {
                            let (ent_c, _, _) = crear_chip_personalizado(
                                commands,
                                pos_mundo,
                                chip.datos.clone(),
                                &config,
                            );
                            commands
                                .entity(ent_c)
                                .insert(Arrastrando { offset: Vec2::ZERO });
                        }
                    }
                }
                return;
            }
        }
    }

    if keys.pressed(KeyCode::Space) 
        || !buttons.just_pressed(MouseButton::Left) 
    {
        return;
    }

    if pos_pantalla.y >= (window.height() - 56.0) {
        return;
    }

    if !en_mano.is_empty() {
        return;
    }

    let (
        mut q_botones, 
        mut q_relojes, 
        q_botones_ver_chip,
        q_flechas_bus,
        q_regulador_bus,
        q_asas_mover,
        q_asas_alto,
    ) = controles_nodos;

    if *estado_app.get() == ModoApp::CreandoLinea {
        if estado_linea.es_bus {
            let n_bus = estado_linea.num_hilos_bus.max(2);

            // A) ¿Clic sobre el anillo circular (PuertoTroncoBus) de otro Conector de Bus?
            for (g_trf, puerto_dest) in q_puertos_bus.iter() {
                let pos_puerto = g_trf.translation().truncate();
                if pos_mundo.distance(pos_puerto) < 16.0 {
                    if let Some(orig_nodo) = estado_linea.origen_entidad {
                        if orig_nodo != puerto_dest.nodo_padre {
                            let n_dest = q_nodos
                                .get(puerto_dest.nodo_padre)
                                .ok()
                                .and_then(|(_, _, _, _, opt_b, _, _)| {
                                    opt_b.map(|b| b.num_hilos)
                                })
                                .unwrap_or(0);

                            if n_dest == n_bus {
                                commands.spawn(CableBus {
                                    origen_nodo: orig_nodo,
                                    destino_nodo: puerto_dest.nodo_padre,
                                    puntos_intermedios: estado_linea
                                        .puntos_intermedios
                                        .clone(),
                                    num_hilos: n_bus,
                                    estados: vec![false; n_bus],
                                });
                            }
                        }
                    }
                    estado_linea.puntos_intermedios.clear();
                    estado_linea.es_bus = false;
                    next_state.set(ModoApp::Edicion);
                    return;
                }
            }

            // B) ¿Clic sobre un CODO existente de otro CableBus (Unión en "T" al tronco del Bus)?
            for (_, cb_existente) in q_cables_bus.iter() {
                if cb_existente.num_hilos != n_bus {
                    continue;
                }
                for (idx_codo, &pos_codo) in cb_existente
                    .puntos_intermedios
                    .iter()
                    .enumerate() 
                {
                    if pos_mundo.distance(pos_codo) < 14.0 {
                        if let Some(orig_nodo) = estado_linea.origen_entidad {
                            let mut nuevos_codos = estado_linea
                                .puntos_intermedios
                                .clone();
                            nuevos_codos.push(pos_codo);
                            for k in (0..idx_codo).rev() {
                                nuevos_codos.push(
                                    cb_existente.puntos_intermedios[k]
                                );
                            }
                            commands.spawn(CableBus {
                                origen_nodo: orig_nodo,
                                destino_nodo: cb_existente.origen_nodo,
                                puntos_intermedios: nuevos_codos,
                                num_hilos: n_bus,
                                estados: vec![false; n_bus],
                            });
                        }
                        estado_linea.puntos_intermedios.clear();
                        estado_linea.es_bus = false;
                        next_state.set(ModoApp::Edicion);
                        return;
                    }
                }
            }
        } else {
            for (ent_destino, g_trf, tipo, opt_bi) in q_conectores.iter() {
                let pos_conector = g_trf.translation().truncate();
                if pos_mundo.distance(pos_conector) < config.rad_clic_con {
                    if *tipo == TipoConector::Entrada || opt_bi.is_some() {
                        if let Some(orig) = estado_linea.origen_entidad {
                            if orig != ent_destino {
                                let duplicado = q_cables
                                    .iter()
                                    .any(|(_, c)| {
                                        c.origen == orig 
                                            && c.destino == ent_destino
                                    });

                                if !duplicado {
                                    commands.spawn(Cable {
                                        origen: orig,
                                        destino: ent_destino,
                                        puntos_intermedios: estado_linea
                                            .puntos_intermedios
                                            .clone(),
                                    });
                                }
                            }
                        }
                        estado_linea.puntos_intermedios.clear();
                        next_state.set(ModoApp::Edicion);
                        return;
                    }
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

    // 0D. PRIORIDAD AGARRADEROS DE SELECCIÓN MÚLTIPLE
    let clic_en_agarradero_seleccionado = q_nodos
        .iter()
        .any(|(_, trf, _, _, _, _, opt_sel)| {
            if opt_sel.is_some() {
                let c = trf.translation.truncate();
                (pos_mundo.x - c.x).abs() <= 46.0 
                    && (pos_mundo.y - c.y).abs() <= 28.0
            } else {
                false
            }
        });

    if clic_en_agarradero_seleccionado {
        for (ent, trf, _, _, _, _, opt_sel) in q_nodos.iter() {
            if opt_sel.is_some() {
                let centro = trf.translation.truncate();
                commands.entity(ent).insert(Arrastrando {
                    offset: centro - pos_mundo,
                });
            }
        }
        return;
    }

    // 0E-0. Clic sobre el anillo circular (PuertoTroncoBus) de un Conector de Bus
    for (g_trf, puerto_bus) in q_puertos_bus.iter() {
        let pos_puerto = g_trf.translation().truncate();
        if pos_mundo.distance(pos_puerto) < 14.0 {
            if let Ok((_, _, _, _, Some(&datos_b), _, _)) = 
                q_nodos.get(puerto_bus.nodo_padre) 
            {
                estado_linea.origen_entidad = Some(puerto_bus.nodo_padre);
                estado_linea.origen_pos = pos_puerto;
                estado_linea.puntos_intermedios.clear();
                estado_linea.es_bus = true;
                estado_linea.num_hilos_bus = datos_b.num_hilos;
                next_state.set(ModoApp::CreandoLinea);
                return;
            }
        }
    }

    // 0E-1. Clic sobre los botones [+] / [-] del Conector de Bus
    for (g_trf, btn_reg) in q_regulador_bus.iter() {
        let pos_btn = g_trf.translation().truncate();
        if pos_mundo.distance(pos_btn) < 10.0 {
            if let Ok((ent_b, trf_b, _, _, Some(&datos_b), _, _)) = 
                q_nodos.get(btn_reg.nodo_padre) 
            {
                let centro = trf_b.translation.truncate();
                let mut nuevos_datos = datos_b;
                let n_actual = nuevos_datos.num_hilos as isize;
                nuevos_datos.num_hilos = 
                    (n_actual + btn_reg.delta).clamp(2, 16) as usize;
                commands.entity(ent_b).despawn();
                crear_conector_bus(
                    commands, 
                    centro, 
                    nuevos_datos, 
                    &config
                );
                return;
            }
        }
    }

    // 0E-2. Clic sobre el cuello/flecha de un Conector de Bus
    for (g_trf, ctrl_flecha) in q_flechas_bus.iter() {
        let pos_flecha = g_trf.translation().truncate();
        if pos_mundo.distance(pos_flecha) < 16.0 {
            if let Ok((ent_b, trf_b, _, _, Some(&datos_b), _, _)) = 
                q_nodos.get(ctrl_flecha.nodo_padre) 
            {
                let centro = trf_b.translation.truncate();
                let mut nuevos_datos = datos_b;
                nuevos_datos.modo = nuevos_datos.modo.siguiente();
                commands.entity(ent_b).despawn();
                crear_conector_bus(
                    commands, 
                    centro, 
                    nuevos_datos, 
                    &config
                );
                return;
            }
        }
    }

    // 0F. Clic sobre Asas de Barras
    for (g_trf, asa_m) in q_asas_mover.iter() {
        let pos_asa = g_trf.translation().truncate();
        if (pos_mundo.x - pos_asa.x).abs() < 45.0 
            && (pos_mundo.y - pos_asa.y).abs() < 13.0 
        {
            arrastre_barra.activo = Some((
                asa_m.0, 
                ModoAjusteBarra::MoverHorizontal
            ));
            return;
        }
    }

    for (g_trf, asa_a) in q_asas_alto.iter() {
        let pos_asa = g_trf.translation().truncate();
        if (pos_mundo.x - pos_asa.x).abs() < 43.0 
            && (pos_mundo.y - pos_asa.y).abs() < 13.0 
        {
            arrastre_barra.activo = Some((
                asa_a.0, 
                ModoAjusteBarra::CambiarAltura
            ));
            return;
        }
    }

    // 1. Prioridad 1: Clic en conector de Salida (o Bidireccional "<->") para empezar un cable normal
    for (ent_conector, g_trf, tipo, opt_bi) in q_conectores.iter() {
        let pos_conector = g_trf.translation().truncate();
        if pos_mundo.distance(pos_conector) < config.rad_clic_con {
            if *tipo == TipoConector::Salida || opt_bi.is_some() {
                estado_linea.origen_entidad = Some(ent_conector);
                estado_linea.origen_pos = pos_conector;
                estado_linea.puntos_intermedios.clear();
                estado_linea.es_bus = false;
                next_state.set(ModoApp::CreandoLinea);
                return; 
            }
        }
    }

    // 1B. Clic sobre un codo de cable normal o de CableBus
    let mut codos_bus_en_punto = Vec::new();
    for (ent_cb, cb) in q_cables_bus.iter() {
        for (idx, &pos_codo) in cb.puntos_intermedios.iter().enumerate() {
            if pos_mundo.distance(pos_codo) < 12.0 {
                codos_bus_en_punto.push((ent_cb, idx));
            }
        }
    }
    if !codos_bus_en_punto.is_empty() {
        codo_arrastrado.seleccion_bus = codos_bus_en_punto;
        return;
    }

    for (ent_cable, cable) in q_cables.iter() {
        for (idx, &pos_codo) in cable.puntos_intermedios.iter().enumerate() {
            if pos_mundo.distance(pos_codo) < 10.0 {
                codo_arrastrado.seleccion = Some((ent_cable, idx));
                return;
            }
        }
    }

    // 2. Clic en cuadrado de cambio de estado (0 / 1)
    for (g_trf, mut boton, mut sprite) in q_botones.iter_mut() {
        let pos_boton = g_trf.translation().truncate();
        if (pos_mundo.x - pos_boton.x).abs() < 11.0 
            && (pos_mundo.y - pos_boton.y).abs() < (config.alto_io / 2.0 + 1.0) 
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

    // 2B. Clic en control de frecuencia del Reloj
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

    // 2C. Clic en botón [VER] de un Chip
    for (g_trf, btn_ver) in q_botones_ver_chip.iter() {
        let pos_btn = g_trf.translation().truncate();
        if (pos_mundo.x - pos_btn.x).abs() < 21.0 
            && (pos_mundo.y - pos_btn.y).abs() < 9.0 
        {
            commands.spawn(PeticionEntrarChip(btn_ver.archivo_chp.clone()));
            return;
        }
    }

    // 3. Prioridad 3: Clic en la zona de agarre de un nodo para arrastrarlo
    let mut nodo_clicado = None;
    for (entidad, trf, nodo, _, opt_bus, _, _) in q_nodos.iter() {
        let centro_nodo = trf.translation.truncate();
        let clic_dentro = if matches!(
            nodo.tipo, 
            TipoPuerta::Interruptor | TipoPuerta::Led
        ) {
            (pos_mundo.x - centro_nodo.x).abs() 
                <= (config.ancho_agarre_io / 2.0 + 4.0)
                && (pos_mundo.y - centro_nodo.y).abs() 
                    <= (config.alto_io / 2.0 + 2.0)
        } else if nodo.tipo == TipoPuerta::ConectorBus {
            let n_hilos = opt_bus.map(|b| b.num_hilos).unwrap_or(8);
            let rot = opt_bus.map(|b| b.rotacion % 4).unwrap_or(0);
            let semialto = (n_hilos as f32 * config.sep_pines_chip) / 2.0;
            let delta = pos_mundo - centro_nodo;
            if rot % 2 == 0 {
                delta.x.abs() <= 14.0 && delta.y.abs() <= semialto
            } else {
                delta.y.abs() <= 14.0 && delta.x.abs() <= semialto
            }
        } else {
            pos_mundo.distance(centro_nodo) < config.rad_clic_nodo
        };

        if clic_dentro {
            nodo_clicado = Some(entidad);
            break;
        }
    }

    if let Some(ent_clic) = nodo_clicado {
        for (ent, _, _, _, _, _, opt_sel) in q_nodos.iter() {
            if opt_sel.is_some() {
                commands.entity(ent).remove::<Seleccionado>();
            }
        }
        commands.entity(ent_clic).insert(Arrastrando {
            offset: Vec2::ZERO,
        });
        return;
    }

    // 4. CLIC EN ZONA VACÍA DEL LIENZO: Deseleccionamos y abrimos Rectángulo de Selección
    for (ent, _, _, _, _, _, opt_sel) in q_nodos.iter() {
        if opt_sel.is_some() {
            commands.entity(ent).remove::<Seleccionado>();
        }
    }
    caja_sel.activa = true;
    caja_sel.inicio = pos_mundo;
    caja_sel.actual = pos_mundo;
}


// ==========================================
// Fichero: src/raton.rs
// Bloque: Bloque 3: Soltar Ratón, Cerrar Rectángulo de Selección, Snap e Imantar a Barras
// ==========================================
pub fn soltar_nodo(
    buttons: Res<ButtonInput<MouseButton>>,
    q_nodos_todos: Query<
        (Entity, &Transform, Option<&DatosConectorBus>), 
        (With<NodoLogico>, Without<Arrastrando>)
    >,
    mut q_arrastrando: Query<
        (Entity, &mut Transform, &NodoLogico), 
        With<Arrastrando>
    >,
    q_barras: Query<
        (&Transform, &Sprite, &BarraChip), 
        Without<Arrastrando>
    >,
    // Agrupamos conectores y cables en una tupla para mantenernos por debajo de los 16 parámetros
    mut consultas_cables: (
        Query<(
            Entity, 
            &GlobalTransform, 
            &TipoConector,
            Option<&ConectorBidireccional>,
        )>,
        Query<(&GlobalTransform, &PuertoTroncoBus)>,
        Query<&mut Cable>,
        Query<&mut CableBus>,
    ),
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    estado_app: Res<State<ModoApp>>,
    mut estados_soltar: (
        ResMut<EstadoCrearLinea>,
        ResMut<CodoArrastrado>,
        ResMut<EstadoArrastreBarra>,
        ResMut<CajaSeleccion>,
        ResMut<NextState<ModoApp>>,
    ),
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if !buttons.just_released(MouseButton::Left) {
        return;
    }

    let (
        ref q_conectores,
        ref q_puertos_bus,
        ref mut q_cables,
        ref mut q_cables_bus,
    ) = consultas_cables;

    let (
        ref mut estado_linea,
        ref mut codo_arrastrado,
        ref mut arrastre_barra,
        ref mut caja_sel,
        ref mut next_state,
    ) = estados_soltar;

    if caja_sel.activa {
        caja_sel.activa = false;
        let min_x = caja_sel.inicio.x.min(caja_sel.actual.x);
        let max_x = caja_sel.inicio.x.max(caja_sel.actual.x);
        let min_y = caja_sel.inicio.y.min(caja_sel.actual.y);
        let max_y = caja_sel.inicio.y.max(caja_sel.actual.y);

        if (max_x - min_x) > 6.0 || (max_y - min_y) > 6.0 {
            for (ent, trf, _) in q_nodos_todos.iter() {
                let p = trf.translation.truncate();
                if p.x >= min_x 
                    && p.x <= max_x 
                    && p.y >= min_y 
                    && p.y <= max_y 
                {
                    commands.entity(ent).insert(Seleccionado);
                }
            }
        }
        return;
    }

    if arrastre_barra.activo.take().is_some() {
        return;
    }

    let Ok(window) = q_windows.single() else { return; };
    if let Some(pos_cur) = window.cursor_position() {
        if pos_cur.y >= (window.height() - 56.0) {
            return;
        }
    }

    if !codo_arrastrado.seleccion_bus.is_empty() {
        let paso_codo = config.paso_snap_codo;
        for (ent_cb, idx) in codo_arrastrado.seleccion_bus.drain(..) {
            if let Ok(mut cb) = q_cables_bus.get_mut(ent_cb) {
                if let Some(codo) = cb.puntos_intermedios.get_mut(idx) {
                    codo.x = (codo.x / paso_codo).round() * paso_codo;
                    codo.y = (codo.y / paso_codo).round() * paso_codo;
                }
            }
        }
        return;
    }

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

    for (entidad, mut trf, nodo) in q_arrastrando.iter_mut() {
        let paso = if matches!(
            nodo.tipo, 
            TipoPuerta::Interruptor 
            | TipoPuerta::Led 
            | TipoPuerta::Display4Bits 
            | TipoPuerta::Display7Seg
            | TipoPuerta::ConectorBus
        ) {
            config.paso_snap_io
        } else {
            config.paso_snap_puerta
        };

        let mut x_final = (trf.translation.x / paso).round() * paso;
        let y_final = (trf.translation.y / paso).round() * paso;

        let mut barra_pegada = None;

        for (trf_barra, spr_barra, tipo_barra) in q_barras.iter() {
            let x_barra = trf_barra.translation.x;
            let alto_b = spr_barra
                .custom_size
                .map(|s| s.y)
                .unwrap_or(800.0);
            let dist_x = (trf.translation.x - x_barra).abs();
            let dentro_alto = y_final.abs() <= (alto_b / 2.0 + 10.0);

            if dentro_alto 
                && *tipo_barra == BarraChip::IzquierdaEntradas 
                && nodo.tipo == TipoPuerta::Interruptor 
                && dist_x < 60.0 
            {
                x_final = x_barra;
                barra_pegada = Some(BarraChip::IzquierdaEntradas);
            }

            if dentro_alto 
                && *tipo_barra == BarraChip::DerechaSalidas 
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

    if *estado_app.get() == ModoApp::CreandoLinea 
        && estado_linea.puntos_intermedios.is_empty() 
    {
        let Ok((cam, cam_trf)) = q_camera.single() else { return; };

        if let Some(pos_pantalla) = window.cursor_position() {
            if let Ok(pos_mundo) = cam.viewport_to_world_2d(
                cam_trf, 
                pos_pantalla
            ) {
                if estado_linea.es_bus {
                    let n_bus = estado_linea.num_hilos_bus.max(2);
                    for (g_trf, puerto_dest) in q_puertos_bus.iter() {
                        let pos_puerto = g_trf.translation().truncate();
                        if pos_mundo.distance(pos_puerto) < 16.0 {
                            if let Some(orig_nodo) = estado_linea.origen_entidad {
                                if orig_nodo != puerto_dest.nodo_padre {
                                    let n_dest = q_nodos_todos
                                        .get(puerto_dest.nodo_padre)
                                        .ok()
                                        .and_then(|(_, _, opt_b)| {
                                            opt_b.map(|b| b.num_hilos)
                                        })
                                        .unwrap_or(0);
                                    if n_dest == n_bus {
                                        commands.spawn(CableBus {
                                            origen_nodo: orig_nodo,
                                            destino_nodo: puerto_dest.nodo_padre,
                                            puntos_intermedios: Vec::new(),
                                            num_hilos: n_bus,
                                            estados: vec![false; n_bus],
                                        });
                                    }
                                    estado_linea.puntos_intermedios.clear();
                                    estado_linea.es_bus = false;
                                    next_state.set(ModoApp::Edicion);
                                    return;
                                }
                            }
                        }
                    }
                } else {
                    for (ent_destino, g_trf, tipo, opt_bi) in q_conectores.iter() {
                        let pos_conector = g_trf.translation().truncate();
                        if pos_mundo.distance(pos_conector) < config.rad_clic_con {
                            if *tipo == TipoConector::Entrada || opt_bi.is_some() {
                                if let Some(orig) = estado_linea.origen_entidad {
                                    if orig != ent_destino {
                                        let duplicado = q_cables
                                            .iter()
                                            .any(|c| {
                                                c.origen == orig 
                                                    && c.destino == ent_destino
                                            });

                                        if !duplicado {
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
    }
}

// --- Bloque 4: Mover Nodo(s), Codos de Cable/Bus, Barras o Actualizar Rectángulo de Selección ---
pub fn mover_nodo_arrastrado(
    keys: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    q_global_trf: Query<&GlobalTransform, Without<CamaraPrincipal>>,
    arrastre_barra: Res<EstadoArrastreBarra>,
    mut caja_sel: ResMut<CajaSeleccion>,
    q_barras_y_nodos: (
        Query<
            (&mut Transform, &Arrastrando),
            Without<BarraChip>,
        >,
        Query<
            (&mut Transform, &mut Sprite, &BarraChip, &Children),
            (Without<Arrastrando>, Without<PegadoABarra>),
        >,
        Query<
            (&mut Transform, &PegadoABarra),
            (Without<BarraChip>, Without<Arrastrando>),
        >,
        Query<
            (
                &mut Transform,
                Option<&AsaAltoBarra>,
                Option<&AsaMoverBarra>,
                Option<&EtiquetaSuperiorBarra>,
            ),
            (
                Without<BarraChip>,
                Without<Arrastrando>,
                Without<PegadoABarra>,
            ),
        >,
    ),
    codo_arrastrado: Res<CodoArrastrado>,
    mut q_cables: Query<&mut Cable>,
    mut q_cables_bus: Query<&mut CableBus>,
) {
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

    if caja_sel.activa {
        caja_sel.actual = pos_mundo;
        return;
    }

    let (
        mut q_arrastrando,
        mut q_barras,
        mut q_pegados,
        mut q_hijos_barra,
    ) = q_barras_y_nodos;

    if let Some((barra_obj, modo_ajuste)) = arrastre_barra.activo {
        for (mut trf_b, mut spr_b, tipo_b, hijos_b) in q_barras.iter_mut() {
            if *tipo_b != barra_obj {
                continue;
            }

            match modo_ajuste {
                ModoAjusteBarra::MoverHorizontal => {
                    let paso = 25.0;
                    let mut nuevo_x = (pos_mundo.x / paso).round() * paso;
                    if barra_obj == BarraChip::IzquierdaEntradas {
                        nuevo_x = nuevo_x.min(-100.0);
                    } else {
                        nuevo_x = nuevo_x.max(100.0);
                    }
                    trf_b.translation.x = nuevo_x;

                    for (mut trf_pin, pegado) in q_pegados.iter_mut() {
                        if pegado.0 == barra_obj {
                            trf_pin.translation.x = nuevo_x;
                        }
                    }
                }
                ModoAjusteBarra::CambiarAltura => {
                    let paso = 50.0;
                    let semialto = (
                        (pos_mundo.y.abs() - 12.0).max(100.0) / paso
                    ).round() * paso;
                    let nuevo_alto = (semialto * 2.0).max(200.0);
                    spr_b.custom_size = Some(Vec2::new(18.0, nuevo_alto));

                    for hijo in hijos_b.iter() {
                        if let Ok((mut trf_h, opt_alto, opt_mover, opt_etq)) = 
                            q_hijos_barra.get_mut(hijo) 
                        {
                            if opt_alto.is_some() {
                                trf_h.translation.y = nuevo_alto / 2.0 + 12.0;
                            } else if opt_mover.is_some() {
                                trf_h.translation.y = -nuevo_alto / 2.0 - 12.0;
                            } else if opt_etq.is_some() {
                                trf_h.translation.y = nuevo_alto / 2.0 + 38.0;
                            }
                        }
                    }
                }
            }
        }
        return;
    }

    for (mut trf, arr) in q_arrastrando.iter_mut() {
        trf.translation.x = pos_mundo.x + arr.offset.x;
        trf.translation.y = pos_mundo.y + arr.offset.y;
    }

    if !codo_arrastrado.seleccion_bus.is_empty() {
        for &(ent_cb, idx) in codo_arrastrado.seleccion_bus.iter() {
            if let Ok(mut cb) = q_cables_bus.get_mut(ent_cb) {
                let pto_ref = if idx > 0 {
                    cb.puntos_intermedios.get(idx - 1).copied()
                } else {
                    None
                };
                if let Some(codo) = cb.puntos_intermedios.get_mut(idx) {
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
        return;
    }

    if let Some((ent_cable, idx)) = codo_arrastrado.seleccion {
        if let Ok(mut cable) = q_cables.get_mut(ent_cable) {
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

// ==========================================
// Fichero: src/raton.rs
// Bloque: Bloque 5: Borrar Nodo, Codo, Cable/CableBus, Insertar Punto (P + Clic Der) o Abrir Menú
// ==========================================
pub fn borrar_con_clic_derecho(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    q_arrastrando: Query<Entity, With<Arrastrando>>,
    q_nodos: Query<
        (Entity, &Transform, Option<&Seleccionado>), 
        With<NodoLogico>
    >,
    // Agrupamos cables, buses y menús en tuplas para no rebasar los 16 parámetros de Bevy
    mut consultas_cables_borrar: (
        Query<(Entity, &mut Cable)>,
        Query<(Entity, &mut CableBus)>,
        Query<(&GlobalTransform, &PuertoTroncoBus)>,
        Query<&GlobalTransform>,
    ),
    q_menus: (
        Query<Entity, With<MenuContextual>>,
        Query<Entity, With<SubMenuContextual>>,
    ),
    estado_app: Res<State<ModoApp>>,
    mut estado_linea: ResMut<EstadoCrearLinea>,
    mut next_state: ResMut<NextState<ModoApp>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }

    if !q_arrastrando.is_empty() {
        for ent in q_arrastrando.iter() {
            commands.entity(ent).despawn();
        }
        return;
    }

    if *estado_app.get() == ModoApp::CreandoLinea {
        if estado_linea.puntos_intermedios.pop().is_none() {
            estado_linea.es_bus = false;
            next_state.set(ModoApp::Edicion);
        }
        return;
    }

    let (ref q_menu, ref q_submenu) = q_menus;
    for ent_menu in q_menu.iter() {
        commands.entity(ent_menu).despawn();
    }
    for ent_sub in q_submenu.iter() {
        commands.entity(ent_sub).despawn();
    }

    let Ok(window) = q_windows.single() else { return; };
    let Ok((cam, cam_trf)) = q_camera.single() else { return; };

    let (
        ref mut q_cables,
        ref mut q_cables_bus,
        ref q_puertos_bus,
        ref q_global_trf,
    ) = consultas_cables_borrar;

    let pos_puerto_de_nodo = |nodo_id: Entity| -> Option<Vec2> {
        for (g_trf, p) in q_puertos_bus.iter() {
            if p.nodo_padre == nodo_id {
                return Some(g_trf.translation().truncate());
            }
        }
        None
    };

    if let Some(pos_pantalla) = window.cursor_position() {
        if let Ok(pos_mundo) = cam.viewport_to_world_2d(
            cam_trf, 
            pos_pantalla
        ) {
            if keys.pressed(KeyCode::KeyP) {
                for (_, mut cb) in q_cables_bus.iter_mut() {
                    if let (Some(p_ori), Some(p_des)) = (
                        pos_puerto_de_nodo(cb.origen_nodo),
                        pos_puerto_de_nodo(cb.destino_nodo),
                    ) {
                        let mut puntos = Vec::new();
                        puntos.push(p_ori);
                        for &codo in cb.puntos_intermedios.iter() {
                            puntos.push(codo);
                        }
                        puntos.push(p_des);

                        for (idx_seg, ventana_pts) in 
                            puntos.windows(2).enumerate() 
                        {
                            let a = ventana_pts[0];
                            let b = ventana_pts[1];
                            let ab = b - a;
                            let len_sq = ab.length_squared();
                            if len_sq > 0.001 {
                                let ap = pos_mundo - a;
                                let t = (ap.dot(ab) / len_sq).clamp(0.0, 1.0);
                                let cercano = a + ab * t;
                                if pos_mundo.distance(cercano) < 14.0 {
                                    let paso = config.paso_snap_codo;
                                    let nuevo_codo = Vec2::new(
                                        (cercano.x / paso).round() * paso,
                                        (cercano.y / paso).round() * paso,
                                    );
                                    cb.puntos_intermedios
                                        .insert(idx_seg, nuevo_codo);
                                    return;
                                }
                            }
                        }
                    }
                }

                for (_, mut cable) in q_cables.iter_mut() {
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

                        for (idx_seg, ventana_pts) in 
                            puntos.windows(2).enumerate() 
                        {
                            let a = ventana_pts[0];
                            let b = ventana_pts[1];
                            let ab = b - a;
                            let len_sq = ab.length_squared();
                            if len_sq > 0.001 {
                                let ap = pos_mundo - a;
                                let t = (ap.dot(ab) / len_sq).clamp(0.0, 1.0);
                                let cercano = a + ab * t;
                                if pos_mundo.distance(cercano) < 10.0 {
                                    let paso = config.paso_snap_codo;
                                    let nuevo_codo = Vec2::new(
                                        (cercano.x / paso).round() * paso,
                                        (cercano.y / paso).round() * paso,
                                    );
                                    cable.puntos_intermedios
                                        .insert(idx_seg, nuevo_codo);
                                    return;
                                }
                            }
                        }
                    }
                }
                return;
            }

            for (ent_nodo, trf, opt_sel) in q_nodos.iter() {
                let centro = trf.translation.truncate();
                if pos_mundo.distance(centro) < config.rad_clic_nodo {
                    if opt_sel.is_some() {
                        for (e_s, _, s) in q_nodos.iter() {
                            if s.is_some() {
                                commands.entity(e_s).despawn();
                            }
                        }
                    } else {
                        commands.entity(ent_nodo).despawn();
                    }
                    return;
                }
            }

            let mut codo_bus_a_borrar = None;
            for (ent_cb, cb) in q_cables_bus.iter() {
                if let Some(idx) = cb
                    .puntos_intermedios
                    .iter()
                    .position(|&c| pos_mundo.distance(c) < 12.0)
                {
                    codo_bus_a_borrar = Some((ent_cb, idx));
                    break;
                }
            }
            if let Some((ent_cb, idx)) = codo_bus_a_borrar {
                if let Ok((_, mut cb)) = q_cables_bus.get_mut(ent_cb) {
                    cb.puntos_intermedios.remove(idx);
                }
                return;
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

            for (ent_cb, cb) in q_cables_bus.iter() {
                if let (Some(p_ori), Some(p_des)) = (
                    pos_puerto_de_nodo(cb.origen_nodo),
                    pos_puerto_de_nodo(cb.destino_nodo),
                ) {
                    let mut puntos = Vec::new();
                    puntos.push(p_ori);
                    for &codo in cb.puntos_intermedios.iter() {
                        puntos.push(codo);
                    }
                    puntos.push(p_des);

                    for ventana_pts in puntos.windows(2) {
                        let a = ventana_pts[0];
                        let b = ventana_pts[1];
                        let ab = b - a;
                        let len_sq = ab.length_squared();
                        if len_sq > 0.001 {
                            let ap = pos_mundo - a;
                            let t = (ap.dot(ab) / len_sq).clamp(0.0, 1.0);
                            let cercano = a + ab * t;
                            if pos_mundo.distance(cercano) < 14.0 {
                                commands.entity(ent_cb).despawn();
                                return;
                            }
                        }
                    }
                }
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