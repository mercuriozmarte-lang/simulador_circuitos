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
};

// ==========================================
// Fichero: src/raton.rs
// Bloque: Bloque 2: Detectar Clic Izquierdo, Asas de Barras y Apilado en Mano
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
        (
            Entity, 
            &Transform, 
            &NodoLogico, 
            Option<&InstanciaChip>,
            Option<&Arrastrando>,
        ), 
        With<NodoLogico>
    >,
    q_conectores: Query<
        (Entity, &GlobalTransform, &TipoConector)
    >,
    q_cables: Query<(Entity, &Cable)>,
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
        Query<(&GlobalTransform, &AsaMoverBarra)>,
        Query<(&GlobalTransform, &AsaAltoBarra)>,
    ),
    mut q_textos_reloj: Query<&mut Text2d, With<TextoReloj>>,
    estado_app: Res<State<ModoApp>>,
    mut estados_edicion: (
        ResMut<EstadoCrearLinea>,
        ResMut<CodoArrastrado>,
        ResMut<EstadoArrastreBarra>,
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
        ref mut commands, 
        ref mut meshes, 
        ref mut materials
    ) = recursos_render;
    let (
        ref mut estado_linea,
        ref mut codo_arrastrado,
        ref mut arrastre_barra,
        ref mut next_state,
    ) = estados_edicion;

    let mut en_mano: Vec<(Entity, TipoPuerta, Option<DatosChip>, f32)> = Vec::new();
    for (ent, _, nodo, opt_chip, opt_arr) in q_nodos.iter() {
        if let Some(arr) = opt_arr {
            en_mano.push((
                ent,
                nodo.tipo,
                opt_chip.map(|c| c.datos.clone()),
                arr.offset_y,
            ));
        }
    }

    if !en_mano.is_empty() {
        if keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowUp) {
            let (_, tipo_base, datos_base, min_offset) = en_mano
                .iter()
                .min_by(|a, b| a.3.partial_cmp(&b.3).unwrap())
                .cloned()
                .unwrap();

            let paso = config.paso_snap_puerta;
            let sep_vertical = if let Some(ref d) = datos_base {
                let max_p = d.pines_entrada.len().max(d.pines_salida.len()).max(1);
                let alto_est = (
                    max_p as f32 * config.sep_pines_chip + 36.0
                ).max(100.0);
                (alto_est / paso).ceil() * paso
            } else {
                100.0
            };

            let nuevo_offset = min_offset - sep_vertical;
            let pos_nueva = Vec2::new(pos_mundo.x, pos_mundo.y + nuevo_offset);

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
                        .insert(Arrastrando { offset_y: nuevo_offset });
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

    if en_mano.is_empty() && keys.just_pressed(KeyCode::KeyD) {
        for (_, trf, nodo, opt_chip, _) in q_nodos.iter() {
            let centro = trf.translation.truncate();
            if pos_mundo.distance(centro) < config.rad_clic_nodo {
                let nueva_pos = Vec2::new(centro.x, centro.y - config.sep_pines_chip);
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
                            Vec2::new(centro.x, centro.y - 100.0),
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
                            Vec2::new(centro.x, centro.y - 150.0), 
                            true, 
                            &config
                        );
                    }
                    TipoPuerta::Display7Seg => {
                        crear_display(
                            commands, 
                            Vec2::new(centro.x, centro.y - 150.0), 
                            false, 
                            &config
                        );
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
                                .insert(Arrastrando { offset_y: 0.0 });
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

    if !en_mano.is_empty() {
        return;
    }

    let (
        mut q_botones, 
        mut q_relojes, 
        q_botones_ver_chip,
        q_asas_mover,
        q_asas_alto,
    ) = controles_nodos;

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

    // 0C. Clic sobre el Asa Inferior (< MOVER >) o Superior (^ ALTO v) de una Barra
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

    for (ent_cable, cable) in q_cables.iter() {
        for (idx, &pos_codo) in cable.puntos_intermedios.iter().enumerate() {
            if pos_mundo.distance(pos_codo) < 10.0 {
                codo_arrastrado.seleccion = Some((ent_cable, idx));
                return;
            }
        }
    }

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

    for (g_trf, btn_ver) in q_botones_ver_chip.iter() {
        let pos_btn = g_trf.translation().truncate();
        if (pos_mundo.x - pos_btn.x).abs() < 21.0 
            && (pos_mundo.y - pos_btn.y).abs() < 9.0 
        {
            commands.spawn(PeticionEntrarChip(btn_ver.archivo_chp.clone()));
            return;
        }
    }

    for (entidad, trf, nodo, _, _) in q_nodos.iter() {
        let centro_nodo = trf.translation.truncate();
        let clic_dentro = if matches!(
            nodo.tipo, 
            TipoPuerta::Interruptor | TipoPuerta::Led
        ) {
            (pos_mundo.x - centro_nodo.x).abs() <= (config.ancho_agarre_io / 2.0 + 4.0)
                && (pos_mundo.y - centro_nodo.y).abs() <= (config.alto_io / 2.0 + 2.0)
        } else {
            pos_mundo.distance(centro_nodo) < config.rad_clic_nodo
        };

        if clic_dentro {
            commands.entity(entidad).insert(Arrastrando { offset_y: 0.0 });
            return; 
        }
    }
}

// ==========================================
// Fichero: src/raton.rs
// Bloque: Bloque 3: Soltar Ratón, Asas de Barra, Snap e Imantar a Barras
// ==========================================
pub fn soltar_nodo(
    buttons: Res<ButtonInput<MouseButton>>,
    mut q_arrastrando: Query<
        (Entity, &mut Transform, &NodoLogico), 
        With<Arrastrando>
    >,
    q_barras: Query<(&Transform, &Sprite, &BarraChip), Without<Arrastrando>>,
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
    mut arrastre_barra: ResMut<EstadoArrastreBarra>,
    mut next_state: ResMut<NextState<ModoApp>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if !buttons.just_released(MouseButton::Left) {
        return;
    }

    // Si estábamos moviendo o estirando una barra con sus asas, terminamos el ajuste
    if arrastre_barra.activo.take().is_some() {
        return;
    }

    let Ok(window) = q_windows.single() else { return; };
    if let Some(pos_cur) = window.cursor_position() {
        if pos_cur.y >= (window.height() - 56.0) {
            return;
        }
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
            TipoPuerta::Interruptor | TipoPuerta::Led
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

// ==========================================
// Fichero: src/raton.rs
// Bloque: Bloque 4: Mover Nodo(s), Codos o Ajustar Barras (Posición X y Altura)
// ==========================================
pub fn mover_nodo_arrastrado(
    keys: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    q_global_trf: Query<&GlobalTransform>,
    arrastre_barra: Res<EstadoArrastreBarra>,
    mut q_barras_y_nodos: (
        Query<(&mut Transform, &Arrastrando), Without<BarraChip>>,
        Query<
            (&mut Transform, &mut Sprite, &BarraChip, &Children),
            Without<Arrastrando>,
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

    let (
        mut q_arrastrando,
        mut q_barras,
        mut q_pegados,
        mut q_hijos_barra,
    ) = q_barras_y_nodos;

    // 1. Si estamos arrastrando un Asa de Barra (< MOVER > o ^ ALTO v)
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

                    // Movemos solidariamente todos los Interruptores/LEDs pegados a esta barra
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

                    // Recolocamos las asas y el cartel en los nuevos extremos de la barra
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

    // 2. Mover nodos normales arrastrados
    for (mut trf, arr) in q_arrastrando.iter_mut() {
        trf.translation.x = pos_mundo.x;
        trf.translation.y = pos_mundo.y + arr.offset_y;
    }

    // 3. Mover codo de cable arrastrado
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
// Bloque: Bloque 5: Borrar Nodo, Codo, Cable, Insertar Punto (P + Clic Der) o Abrir Menú
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

    // Si llevábamos una columna de chips en la mano, el clic derecho cancela su colocación
    if !q_arrastrando.is_empty() {
        for ent in q_arrastrando.iter() {
            commands.entity(ent).despawn();
        }
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
            // 0. MODO INSERTAR ESQUINA (Tecla P + Clic Derecho sobre un tramo de cable):
            //    Inserta un nuevo punto intermedio en el tramo tocado sin borrar nada.
            if keys.pressed(KeyCode::KeyP) {
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

                        for (idx_seg, ventana_pts) in puntos.windows(2).enumerate() {
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
                                    cable.puntos_intermedios.insert(idx_seg, nuevo_codo);
                                    return;
                                }
                            }
                        }
                    }
                }
                return;
            }

            // 1. Borrar un nodo si el ratón está encima (y no se pulsa P)
            for (ent_nodo, trf) in q_nodos.iter() {
                let centro = trf.translation.truncate();
                if pos_mundo.distance(centro) < config.rad_clic_nodo {
                    commands.entity(ent_nodo).despawn();
                    return;
                }
            }

            // 2A. Borrar SOLO un codo si hace clic derecho sobre la esquina
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

            // 2B. Borrar el cable entero si hace clic derecho sobre un tramo recto
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