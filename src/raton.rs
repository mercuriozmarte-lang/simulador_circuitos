// ==========================================
// Fichero: src/raton.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::componentes::*;
use crate::configuracion::ConfigApp;
use crate::archivo::{
    abrir_dialogo_archivo, 
    abrir_selector_carga,
};
use crate::{
    crear_puerta, 
    crear_interruptor, 
    crear_led,
    crear_reloj,
    crear_display,
    crear_barras_chip,
};

// ==========================================
// Fichero: src/raton.rs
// Bloque: Bloque 2A: Gestión de Hover y Clics en el Menú Contextual
// ==========================================
pub fn gestionar_clic_menu(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    q_nodos: Query<Entity, With<NodoLogico>>,
    q_cables: Query<Entity, With<Cable>>,
    q_barras: Query<Entity, With<BarraChip>>,
    q_menu: Query<(Entity, &Transform, &MenuContextual)>,
    q_submenu: Query<(Entity, &SubMenuContextual)>,
    mut q_botones_menu: Query<(&BotonMenu, &mut Sprite)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    // Si no hay menú abierto, salimos
    let Ok((ent_menu, trf_menu, menu)) = q_menu.single() else {
        return;
    };

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

    let mut accion_bajo_cursor = None;
    let mitad_ancho = 65.0;
    let mitad_alto = 16.0;

    for (btn_menu, mut sprite) in q_botones_menu.iter_mut() {
        let pos_btn = btn_menu.centro_mundo;
        let dentro_x = (pos_mundo.x - pos_btn.x).abs() < mitad_ancho;
        let dentro_y = (pos_mundo.y - pos_btn.y).abs() < mitad_alto;

        if dentro_x && dentro_y {
            accion_bajo_cursor = Some(btn_menu.accion);
            sprite.color = Color::srgb(0.38, 0.48, 0.62);
        } else {
            sprite.color = Color::srgb(0.24, 0.28, 0.36);
        }
    }

    // 1. HOVER AUTOMÁTICO: Desplegar submenú
    if let Some(AccionMenu::AbrirCategoria(cat)) = accion_bajo_cursor {
        let mut ya_abierto = false;
        for (ent_sub, sub) in q_submenu.iter() {
            if sub.categoria == cat {
                ya_abierto = true;
            } else {
                commands.entity(ent_sub).despawn();
            }
        }

        if !ya_abierto {
            let pos_base = trf_menu.translation.truncate();
            abrir_submenu(&mut commands, pos_base, cat);
        }
    }

    // 2. CLIC IZQUIERDO
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    if let Some(accion) = accion_bajo_cursor {
        match accion {
            AccionMenu::AbrirCategoria(cat) => {
                for (ent_sub, sub) in q_submenu.iter() {
                    if sub.categoria != cat {
                        commands.entity(ent_sub).despawn();
                    }
                }
                let pos_base = trf_menu.translation.truncate();
                abrir_submenu(&mut commands, pos_base, cat);
                return;
            }
            AccionMenu::CrearNodo(tipo) => {
                let cuadricula = 50.0;
                let pos_snap = Vec2::new(
                    (menu.pos_creacion.x / cuadricula).round() 
                        * cuadricula,
                    (menu.pos_creacion.y / cuadricula).round() 
                        * cuadricula,
                );

                match tipo {
                    TipoPuerta::And 
                    | TipoPuerta::Or 
                    | TipoPuerta::Not 
                    | TipoPuerta::Nand 
                    | TipoPuerta::Nor 
                    | TipoPuerta::Xor 
                    | TipoPuerta::Xnor => {
                        crear_puerta(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            pos_snap, 
                            tipo, 
                            &config
                        );
                    }
                    TipoPuerta::Interruptor => {
                        crear_interruptor(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            pos_snap,
                            false,
                            &config,
                        );
                    }
                    TipoPuerta::Led => {
                        crear_led(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            pos_snap,
                            &config,
                        );
                    }
                    TipoPuerta::Reloj => {
                        crear_reloj(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            pos_snap,
                            1,
                            &config,
                        );
                    }
                    TipoPuerta::Display4Bits => {
                        crear_display(
                            &mut commands,
                            pos_snap,
                            true,
                            &config,
                        );
                    }
                    TipoPuerta::Display7Seg => {
                        crear_display(
                            &mut commands,
                            pos_snap,
                            false,
                            &config,
                        );
                    }
                    TipoPuerta::Chip => {}
                }
            }
            AccionMenu::AlternarBarrasChip => {
                if q_barras.is_empty() {
                    crear_barras_chip(&mut commands);
                } else {
                    for ent_b in q_barras.iter() {
                        commands.entity(ent_b).despawn();
                    }
                }
            }
            AccionMenu::GuardarChip => {
                // true = Guardar como Chip (.chp)
                abrir_dialogo_archivo(&mut commands, true);
            }
            AccionMenu::GuardarCircuito => {
                // false = Guardar como Circuito (.cir)
                abrir_dialogo_archivo(&mut commands, false);
            }
            AccionMenu::CargarCircuito => {
                abrir_selector_carga(&mut commands);
            }
            AccionMenu::LimpiarTodo => {
                for ent_n in q_nodos.iter() {
                    commands.entity(ent_n).despawn();
                }
                for ent_c in q_cables.iter() {
                    commands.entity(ent_c).despawn();
                }
            }
        }
    }

    commands.entity(ent_menu).despawn();
    for (ent_sub, _) in q_submenu.iter() {
        commands.entity(ent_sub).despawn();
    }
}

// --- Bloque 2B: Clic en Nodos, Conectores y Codos de Cable ---
pub fn detectar_clic_nodo(
    buttons: Res<ButtonInput<MouseButton>>,
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
            With<SelectorCarga>
        )>
    >,
    q_nodos: Query<(Entity, &Transform), With<NodoLogico>>,
    q_conectores: Query<
        (Entity, &GlobalTransform, &TipoConector)
    >,
    q_cables: Query<(Entity, &Cable)>,
    mut q_botones: Query<
        (&GlobalTransform, &mut BotonInterruptor, &mut Sprite),
        Without<ControlReloj>,
    >,
    mut q_relojes: Query<
        (&GlobalTransform, &mut ControlReloj, &mut Sprite, &Children),
        Without<BotonInterruptor>,
    >,
    mut q_textos_reloj: Query<&mut Text2d, With<TextoReloj>>,
    estado_app: Res<State<ModoApp>>,
    mut estado_linea: ResMut<EstadoCrearLinea>,
    mut codo_arrastrado: ResMut<CodoArrastrado>,
    mut next_state: ResMut<NextState<ModoApp>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

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

    // MODO CREANDO LÍNEA: Añadir codo o finalizar cable en una Entrada
    if *estado_app.get() == ModoApp::CreandoLinea {
        // 1. ¿Ha hecho clic sobre un conector de Entrada para terminar el cable?
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

        // 2. Si clicó en el lienzo vacío, añadimos un codo ajustado a 25px
        let paso_codo = 25.0;
        let codo_snap = Vec2::new(
            (pos_mundo.x / paso_codo).round() * paso_codo,
            (pos_mundo.y / paso_codo).round() * paso_codo,
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

    // 1B. NUEVO: Clic sobre una esquina (codo) de un cable ya creado para moverla
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

    // 3. Prioridad 3: Clic en el cuerpo del nodo para arrastrarlo
    for (entidad, trf) in q_nodos.iter() {
        let centro_nodo = trf.translation.truncate();
        if pos_mundo.distance(centro_nodo) < config.rad_clic_nodo {
            commands.entity(entidad).insert(Arrastrando);
            return; 
        }
    }
}

// --- Bloque 3: Soltar Ratón, Codos e Imantar a Barras de Chip ---
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

    // 0. NUEVO: Si estábamos arrastrando el codo de un cable, lo encajamos a 25px y soltamos
    if let Some((ent_cable, idx)) = codo_arrastrado.seleccion.take() {
        if let Ok(mut cable) = q_cables.get_mut(ent_cable) {
            if let Some(codo) = cable.puntos_intermedios.get_mut(idx) {
                let paso_codo = 25.0;
                codo.x = (codo.x / paso_codo).round() * paso_codo;
                codo.y = (codo.y / paso_codo).round() * paso_codo;
            }
        }
        return;
    }

    // 1. Soltar nodos arrastrados y comprobar si se pegan a una Barra de Chip
    for (entidad, mut trf, nodo) in q_arrastrando.iter_mut() {
        let cuadricula = 50.0;
        let mut x_final = 
            (trf.translation.x / cuadricula).round() * cuadricula;
        let y_final = 
            (trf.translation.y / cuadricula).round() * cuadricula;

        let mut barra_pegada = None;

        for (trf_barra, tipo_barra) in q_barras.iter() {
            let x_barra = trf_barra.translation.x;
            let dist_x = (trf.translation.x - x_barra).abs();

            // Si acerca un Interruptor a la barra izquierda (< 60px)
            if *tipo_barra == BarraChip::IzquierdaEntradas 
                && nodo.tipo == TipoPuerta::Interruptor 
                && dist_x < 60.0 
            {
                x_final = x_barra;
                barra_pegada = Some(BarraChip::IzquierdaEntradas);
            }

            // Si acerca un LED a la barra derecha (< 60px)
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

    // 2. Si el usuario arrastró y soltó directamente sobre una Entrada (sin codos),
    // también permitimos cerrar el cable al soltar el botón
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

// --- Bloque 4: Mover Nodo o Codo Arrastrado ---
pub fn mover_nodo_arrastrado(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
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
            // 1. Mover puertas o nodos arrastrados
            for mut trf in q_arrastrando.iter_mut() {
                trf.translation.x = pos_mundo.x;
                trf.translation.y = pos_mundo.y;
            }

            // 2. NUEVO: Mover el codo seleccionado en tiempo real
            if let Some((ent_cable, idx)) = codo_arrastrado.seleccion {
                if let Ok(mut cable) = q_cables.get_mut(ent_cable) {
                    if let Some(codo) = cable.puntos_intermedios.get_mut(idx) {
                        *codo = pos_mundo;
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

    // Si estábamos tirando un cable con codos, el clic derecho deshace el último codo
    // o cancela el cable si ya no quedan codos
    if *estado_app.get() == ModoApp::CreandoLinea {
        if estado_linea.puntos_intermedios.pop().is_none() {
            next_state.set(ModoApp::Edicion);
        }
        return;
    }

    // Si ya había un menú o submenú abierto, los cerramos primero
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
            // 1. Intentamos borrar una puerta lógica si el ratón está encima
            for (ent_nodo, trf) in q_nodos.iter() {
                let centro = trf.translation.truncate();
                if pos_mundo.distance(centro) < config.rad_clic_nodo {
                    commands.entity(ent_nodo).despawn();
                    return;
                }
            }

            // 2A. NUEVO: Si hace clic derecho exactamente sobre un codo (< 10px),
            // borramos SOLO ese codo sin destruir el cable entero
            for (_, mut cable) in q_cables.iter_mut() {
                if let Some(idx) = cable
                    .puntos_intermedios
                    .iter()
                    .position(|&c| pos_mundo.distance(c) < 10.0)
                {
                    cable.puntos_intermedios.remove(idx);
                    return;
                }
            }

            // 2B. Si hace clic derecho sobre un tramo recto del cable, borramos el cable
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

            // 3. Si estamos en vacío, abrimos el menú principal compacto
            abrir_menu_contextual(&mut commands, pos_mundo);
        }
    }
}

// ==========================================
// Fichero: src/raton.rs
// Bloque: Bloque 6: Creador del Menú Principal y Submenús
// ==========================================
fn abrir_menu_contextual(
    commands: &mut Commands, 
    pos: Vec2
) {
    let categorias = [
        ("Puertas  >>", AccionMenu::AbrirCategoria(CategoriaMenu::Puertas)),
        ("Entradas >>", AccionMenu::AbrirCategoria(CategoriaMenu::Entradas)),
        ("Salidas  >>", AccionMenu::AbrirCategoria(CategoriaMenu::Salidas)),
        ("Chip     >>", AccionMenu::AbrirCategoria(CategoriaMenu::Chip)),
        ("Archivo  >>", AccionMenu::AbrirCategoria(CategoriaMenu::Archivo)),
    ];

    let ancho_btn = 130.0;
    let alto_btn = 32.0;
    let separacion = 36.0;
    let total_alto = (categorias.len() as f32) * separacion + 10.0;

    commands.spawn((
        Sprite {
            color: Color::srgb(0.12, 0.12, 0.15),
            custom_size: Some(Vec2::new(ancho_btn + 12.0, total_alto)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 5.0),
        MenuContextual { pos_creacion: pos },
    ))
    .with_children(|padre| {
        let inicio_y = 
            ((categorias.len() as f32 - 1.0) * separacion) / 2.0;

        for (i, (texto, accion)) in categorias.iter().enumerate() {
            let y = inicio_y - (i as f32 * separacion);
            let centro_mundo = Vec2::new(pos.x, pos.y + y);

            padre.spawn((
                Sprite {
                    color: Color::srgb(0.24, 0.28, 0.36),
                    custom_size: Some(Vec2::new(ancho_btn, alto_btn)),
                    ..default()
                },
                Transform::from_xyz(0.0, y, 0.1),
                BotonMenu { 
                    accion: *accion,
                    centro_mundo,
                },
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text2d::new(*texto),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(15.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 0.0, 0.1),
                ));
            });
        }
    });
}

fn abrir_submenu(
    commands: &mut Commands,
    pos_menu_principal: Vec2,
    categoria: CategoriaMenu,
) {
    let opciones: Vec<(&str, AccionMenu)> = match categoria {
        CategoriaMenu::Puertas => vec![
            ("Puerta AND", AccionMenu::CrearNodo(TipoPuerta::And)),
            ("Puerta OR", AccionMenu::CrearNodo(TipoPuerta::Or)),
            ("Puerta NOT", AccionMenu::CrearNodo(TipoPuerta::Not)),
            ("Puerta NAND", AccionMenu::CrearNodo(TipoPuerta::Nand)),
            ("Puerta NOR", AccionMenu::CrearNodo(TipoPuerta::Nor)),
            ("Puerta XOR", AccionMenu::CrearNodo(TipoPuerta::Xor)),
            ("Puerta XNOR", AccionMenu::CrearNodo(TipoPuerta::Xnor)),
        ],
        CategoriaMenu::Entradas => vec![
            ("Interruptor", AccionMenu::CrearNodo(TipoPuerta::Interruptor)),
            ("Reloj (Clock)", AccionMenu::CrearNodo(TipoPuerta::Reloj)),
        ],
        CategoriaMenu::Salidas => vec![
            ("LED Salida", AccionMenu::CrearNodo(TipoPuerta::Led)),
            ("Display 4-Bits", AccionMenu::CrearNodo(TipoPuerta::Display4Bits)),
            ("Display 7-Seg", AccionMenu::CrearNodo(TipoPuerta::Display7Seg)),
        ],
        CategoriaMenu::Chip => vec![
            ("Barras I/O", AccionMenu::AlternarBarrasChip),
            ("Guardar .chp", AccionMenu::GuardarChip),
        ],
        CategoriaMenu::Archivo => vec![
            ("Guardar .cir", AccionMenu::GuardarCircuito),
            ("Cargar .cir", AccionMenu::CargarCircuito),
            ("Limpiar Todo", AccionMenu::LimpiarTodo),
        ],
    };

    let ancho_btn = 130.0;
    let alto_btn = 32.0;
    let separacion = 36.0;
    let total_alto = (opciones.len() as f32) * separacion + 10.0;

    let pos_sub = Vec2::new(
        pos_menu_principal.x + ancho_btn + 14.0,
        pos_menu_principal.y,
    );

    commands.spawn((
        Sprite {
            color: Color::srgb(0.15, 0.16, 0.20),
            custom_size: Some(Vec2::new(ancho_btn + 12.0, total_alto)),
            ..default()
        },
        Transform::from_xyz(pos_sub.x, pos_sub.y, 5.5),
        SubMenuContextual { categoria },
    ))
    .with_children(|padre| {
        let inicio_y = 
            ((opciones.len() as f32 - 1.0) * separacion) / 2.0;

        for (i, (texto, accion)) in opciones.iter().enumerate() {
            let y = inicio_y - (i as f32 * separacion);
            let centro_mundo = Vec2::new(pos_sub.x, pos_sub.y + y);

            padre.spawn((
                Sprite {
                    color: Color::srgb(0.24, 0.28, 0.36),
                    custom_size: Some(Vec2::new(ancho_btn, alto_btn)),
                    ..default()
                },
                Transform::from_xyz(0.0, y, 0.1),
                BotonMenu { 
                    accion: *accion,
                    centro_mundo,
                },
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text2d::new(*texto),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(15.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 0.0, 0.1),
                ));
            });
        }
    });
}