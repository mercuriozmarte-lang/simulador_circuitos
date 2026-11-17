// ==========================================
// Fichero: src/menus.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::componentes::*;
use crate::configuracion::ConfigApp;
use crate::archivo::{
    abrir_dialogo_archivo,
    abrir_selector_carga,
    listar_archivos_por_ext,
    leer_datos_chip_desde_chp,
    PeticionSalirChip,
};
use crate::{
    crear_puerta,
    crear_interruptor,
    crear_led,
    crear_reloj,
    crear_display,
    crear_barras_chip,
    crear_chip_personalizado,
};

// ==========================================
// Fichero: src/menus.rs
// Bloque: Bloque 2: Gestión de Hover y Clics en el Menú Contextual
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
    q_cables_bus: Query<Entity, With<CableBus>>,
    q_barras: Query<Entity, With<BarraChip>>,
    q_menu: Query<(Entity, &Transform, &MenuContextual)>,
    q_submenu: Query<(Entity, &SubMenuContextual)>,
    mut q_botones_menu: Query<(&BotonMenu, &mut Sprite)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
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

    let pos_cam = cam_trf.translation().truncate();

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
            AccionMenu::CrearConectorBus(num_hilos) => {
                let (ent_bus, _, _) = crate::crear_conector_bus(
                    &mut commands,
                    pos_mundo,
                    DatosConectorBus {
                        num_hilos,
                        rotacion: 0,
                        modo: ModoDireccionBus::HaciaBus,
                    },
                    &config,
                );
                commands
                    .entity(ent_bus)
                    .insert(Arrastrando { offset: Vec2::ZERO });
            }
            AccionMenu::CrearNodo(tipo) => {
                let paso = if matches!(
                    tipo, 
                    TipoPuerta::Interruptor 
                    | TipoPuerta::Led
                    | TipoPuerta::ConectorBus
                ) {
                    config.paso_snap_io
                } else {
                    config.paso_snap_puerta
                };

                let pos_snap = Vec2::new(
                    (menu.pos_creacion.x / paso).round() * paso,
                    (menu.pos_creacion.y / paso).round() * paso,
                );

                match tipo {
                    TipoPuerta::And 
                    | TipoPuerta::Or 
                    | TipoPuerta::Not 
                    | TipoPuerta::Nand 
                    | TipoPuerta::Nor 
                    | TipoPuerta::Xor 
                    | TipoPuerta::Xnor
                    | TipoPuerta::TriState => {
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
                            "",
                            &config,
                        );
                    }
                    TipoPuerta::Led => {
                        crear_led(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            pos_snap,
                            "",
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
                    TipoPuerta::ConectorBus => {
                        crate::crear_conector_bus(
                            &mut commands,
                            pos_snap,
                            DatosConectorBus {
                                num_hilos: 8,
                                rotacion: 0,
                                modo: ModoDireccionBus::HaciaBus,
                            },
                            &config,
                        );
                    }
                    TipoPuerta::Chip => {}
                }
            }
            AccionMenu::AlternarBarrasChip => {
                if q_barras.is_empty() {
                    crear_barras_chip(&mut commands, &config);
                } else {
                    for ent_b in q_barras.iter() {
                        commands.entity(ent_b).despawn();
                    }
                }
            }
            AccionMenu::GuardarChip => {
                abrir_dialogo_archivo(&mut commands, pos_cam, true);
            }
            AccionMenu::GuardarCircuito => {
                abrir_dialogo_archivo(&mut commands, pos_cam, false);
            }
            AccionMenu::CargarCircuito => {
                abrir_selector_carga(&mut commands, pos_cam);
            }
            AccionMenu::LimpiarTodo => {
                for ent_n in q_nodos.iter() {
                    commands.entity(ent_n).despawn();
                }
                for ent_c in q_cables.iter() {
                    commands.entity(ent_c).despawn();
                }
                for ent_cb in q_cables_bus.iter() {
                    commands.entity(ent_cb).despawn();
                }
            }
        }
    }

    commands.entity(ent_menu).despawn();
    for (ent_sub, _) in q_submenu.iter() {
        commands.entity(ent_sub).despawn();
    }
}

// ==========================================
// Fichero: src/menus.rs
// Bloque: Bloque 3: Constructores del Menú Contextual y Submenús
// ==========================================
pub fn abrir_menu_contextual(
    commands: &mut Commands, 
    pos_mundo: Vec2
) {
    let items_principales: [(&str, AccionMenu); 7] = [
        ("Puertas     >>", AccionMenu::AbrirCategoria(CategoriaMenu::Puertas)),
        ("Entradas    >>", AccionMenu::AbrirCategoria(CategoriaMenu::Entradas)),
        ("Salidas     >>", AccionMenu::AbrirCategoria(CategoriaMenu::Salidas)),
        ("Bus         >>", AccionMenu::AbrirCategoria(CategoriaMenu::Bus)),
        ("Chip        >>", AccionMenu::AbrirCategoria(CategoriaMenu::Chip)),
        ("Archivo     >>", AccionMenu::AbrirCategoria(CategoriaMenu::Archivo)),
        ("Limpiar Todo",   AccionMenu::LimpiarTodo),
    ];

    let ancho_boton = 130.0;
    let alto_boton = 32.0;
    let separacion = 36.0;
    let num_items = items_principales.len() as f32;
    let total_alto = num_items * separacion + 12.0;

    commands.spawn((
        Sprite {
            color: Color::srgb(0.14, 0.16, 0.22),
            custom_size: Some(Vec2::new(ancho_boton + 16.0, total_alto)),
            ..default()
        },
        Transform::from_xyz(pos_mundo.x, pos_mundo.y, 10.0),
        MenuContextual {
            pos_creacion: pos_mundo,
        },
    ))
    .with_children(|padre| {
        let inicio_y = ((num_items - 1.0) * separacion) / 2.0;

        for (i, (etiqueta, accion)) in items_principales.iter().enumerate() {
            let offset_y = inicio_y - (i as f32 * separacion);
            let centro_btn = Vec2::new(
                pos_mundo.x, 
                pos_mundo.y + offset_y
            );

            padre.spawn((
                Sprite {
                    color: Color::srgb(0.24, 0.28, 0.36),
                    custom_size: Some(Vec2::new(ancho_boton, alto_boton)),
                    ..default()
                },
                Transform::from_xyz(0.0, offset_y, 0.1),
                BotonMenu {
                    accion: *accion,
                    centro_mundo: centro_btn,
                },
            ))
            .with_children(|boton| {
                boton.spawn((
                    Text2d::new(*etiqueta),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(14.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 0.0, 0.2),
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
            ("Puerta AND",      AccionMenu::CrearNodo(TipoPuerta::And)),
            ("Puerta OR",       AccionMenu::CrearNodo(TipoPuerta::Or)),
            ("Puerta NOT",      AccionMenu::CrearNodo(TipoPuerta::Not)),
            ("Puerta NAND",     AccionMenu::CrearNodo(TipoPuerta::Nand)),
            ("Puerta NOR",      AccionMenu::CrearNodo(TipoPuerta::Nor)),
            ("Puerta XOR",      AccionMenu::CrearNodo(TipoPuerta::Xor)),
            ("Puerta XNOR",     AccionMenu::CrearNodo(TipoPuerta::Xnor)),
            ("Buffer TriState", AccionMenu::CrearNodo(TipoPuerta::TriState)),
        ],
        CategoriaMenu::Entradas => vec![
            ("Interruptor",  AccionMenu::CrearNodo(TipoPuerta::Interruptor)),
            ("Reloj (CLK)",  AccionMenu::CrearNodo(TipoPuerta::Reloj)),
        ],
        CategoriaMenu::Salidas => vec![
            ("LED Simple",    AccionMenu::CrearNodo(TipoPuerta::Led)),
            ("Display 4-Bit", AccionMenu::CrearNodo(TipoPuerta::Display4Bits)),
            ("Display 7-Seg", AccionMenu::CrearNodo(TipoPuerta::Display7Seg)),
        ],
        CategoriaMenu::Bus => vec![
            ("Conector Bus 4", AccionMenu::CrearConectorBus(4)),
            ("Conector Bus 8", AccionMenu::CrearConectorBus(8)),
        ],
        CategoriaMenu::Chip => vec![
            ("Barras I/O",   AccionMenu::AlternarBarrasChip),
            ("Guardar .chp", AccionMenu::GuardarChip),
        ],
        CategoriaMenu::Archivo => vec![
            ("Guardar .cir", AccionMenu::GuardarCircuito),
            ("Cargar .cir",  AccionMenu::CargarCircuito),
        ],
    };

    let ancho_boton = 130.0;
    let alto_boton = 32.0;
    let separacion = 36.0;
    let num_items = opciones.len() as f32;
    let total_alto = num_items * separacion + 12.0;

    let pos_sub = Vec2::new(
        pos_menu_principal.x + 150.0,
        pos_menu_principal.y,
    );

    commands.spawn((
        Sprite {
            color: Color::srgb(0.14, 0.16, 0.22),
            custom_size: Some(Vec2::new(ancho_boton + 16.0, total_alto)),
            ..default()
        },
        Transform::from_xyz(pos_sub.x, pos_sub.y, 11.0),
        SubMenuContextual { categoria },
    ))
    .with_children(|padre| {
        let inicio_y = ((num_items - 1.0) * separacion) / 2.0;

        for (i, (etiqueta, accion)) in opciones.iter().enumerate() {
            let offset_y = inicio_y - (i as f32 * separacion);
            let centro_btn = Vec2::new(
                pos_sub.x, 
                pos_sub.y + offset_y
            );

            padre.spawn((
                Sprite {
                    color: Color::srgb(0.24, 0.28, 0.36),
                    custom_size: Some(Vec2::new(ancho_boton, alto_boton)),
                    ..default()
                },
                Transform::from_xyz(0.0, offset_y, 0.1),
                BotonMenu {
                    accion: *accion,
                    centro_mundo: centro_btn,
                },
            ))
            .with_children(|boton| {
                boton.spawn((
                    Text2d::new(*etiqueta),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(14.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 0.0, 0.2),
                ));
            });
        }
    });
}

// ==========================================
// Fichero: src/menus.rs
// Bloque: Bloque 4: Construcción de las Barras Flotantes UI (Inferior con Scroll y Superior)
// ==========================================
pub fn reconstruir_barra_inferior_chips(
    commands: &mut Commands,
    q_barra_previa: &Query<Entity, With<BarraInferiorChips>>,
    indice_inicio: usize,
) {
    for ent in q_barra_previa.iter() {
        commands.entity(ent).despawn();
    }

    let archivos_chp = listar_archivos_por_ext("chips", "chp");
    let max_visibles = 8usize;
    let total = archivos_chp.len();
    let inicio = if total > 0 {
        indice_inicio.min(total.saturating_sub(1))
    } else {
        0
    };
    let fin = (inicio + max_visibles).min(total);

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(20.0),
            height: Val::Px(42.0),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
            column_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.94)),
        BarraInferiorChips,
    ))
    .with_children(|barra| {
        let mut cursor_x = 30.0;

        let w_flecha = 32.0;
        barra.spawn((
            Node {
                width: Val::Px(w_flecha),
                height: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.16, 0.20, 0.28)),
            BotonScrollBarra {
                delta: -1,
                x_min: cursor_x,
                x_max: cursor_x + w_flecha,
            },
        ))
        .with_children(|b| {
            b.spawn((
                Text::new("<"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(15.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
        cursor_x += w_flecha + 8.0;

        if archivos_chp.is_empty() {
            barra.spawn((
                Text::new("Carpeta chips/ vacía (Usa Chip >> Guardar .chp)"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(12.5),
                    ..default()
                },
                TextColor(Color::srgb(0.60, 0.65, 0.75)),
            ));
        } else {
            for ruta in &archivos_chp[inicio..fin] {
                let nombre_mostrar = std::path::Path::new(ruta)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("CHIP")
                    .to_uppercase();

                let etiqueta_btn = format!("+{}", nombre_mostrar);
                let ancho_btn = (
                    etiqueta_btn.chars().count() as f32 * 8.5 + 22.0
                ).max(74.0);

                let x_min = cursor_x;
                let x_max = cursor_x + ancho_btn;
                cursor_x = x_max + 8.0;

                barra.spawn((
                    Node {
                        width: Val::Px(ancho_btn),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.20, 0.34, 0.50)),
                    BotonChipInferior {
                        archivo_chp: ruta.clone(),
                        x_min,
                        x_max,
                    },
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(etiqueta_btn),
                        TextFont {
                            font_size: bevy::text::FontSize::Px(12.5),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        }

        barra.spawn((
            Node {
                width: Val::Px(w_flecha),
                height: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.16, 0.20, 0.28)),
            BotonScrollBarra {
                delta: 1,
                x_min: cursor_x,
                x_max: cursor_x + w_flecha,
            },
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(">"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(15.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
    });
}

pub fn reconstruir_barra_superior_inspeccion(
    commands: &mut Commands,
    q_barra_sup: &Query<Entity, With<BarraSuperiorInspeccion>>,
    pila: &PilaInspeccion,
) {
    for ent in q_barra_sup.iter() {
        commands.entity(ent).despawn();
    }

    if pila.niveles.is_empty() {
        return;
    }

    let ruta_actual: Vec<String> = pila
        .niveles
        .iter()
        .map(|(arch, _)| {
            std::path::Path::new(arch)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(arch.as_str())
                .to_uppercase()
        })
        .collect();

    let texto_ruta = format!(
        "INSPECCIONANDO: {}   (Usa [Ctrl+Z] para deshacer cambios)",
        ruta_actual.join(" > ")
    );

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(20.0),
            height: Val::Px(42.0),
            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
            column_gap: Val::Px(16.0),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.12, 0.14, 0.20, 0.95)),
        BarraSuperiorInspeccion,
    ))
    .with_children(|barra| {
        barra.spawn((
            Node {
                width: Val::Px(160.0),
                height: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.65, 0.25, 0.25)),
            BotonVolverInspeccion,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("<< VOLVER AL CIRCUITO"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        barra.spawn((
            Text::new(texto_ruta),
            TextFont {
                font_size: bevy::text::FontSize::Px(13.5),
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.85, 0.45)),
        ));
    });
}

// ==========================================
// Fichero: src/menus.rs
// Bloque: Bloque 5: Interacción con las Barras Flotantes UI (Scroll y Multi-Chip)
// ==========================================
pub fn gestionar_clic_barras_ui(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<&Transform, With<CamaraPrincipal>>,
    q_barra_inf: Query<Entity, With<BarraInferiorChips>>,
    mut scroll_barra: ResMut<DesplazamientoBarraChips>,
    q_botones_ui: (
        Query<
            (&BotonChipInferior, &mut BackgroundColor),
            (Without<BotonScrollBarra>, Without<BotonVolverInspeccion>),
        >,
        Query<
            (&BotonScrollBarra, &mut BackgroundColor),
            (Without<BotonChipInferior>, Without<BotonVolverInspeccion>),
        >,
        Query<
            &mut BackgroundColor,
            (
                With<BotonVolverInspeccion>,
                Without<BotonChipInferior>,
                Without<BotonScrollBarra>,
            ),
        >,
    ),
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    let Ok(window) = q_windows.single() else { return; };
    let Some(pos_cursor) = window.cursor_position() else {
        return;
    };

    let (
        mut q_botones_inf, 
        mut q_botones_scroll, 
        mut q_boton_volver
    ) = q_botones_ui;

    for mut color_volver in q_boton_volver.iter_mut() {
        let en_y_sup = pos_cursor.y >= 10.0 && pos_cursor.y <= 54.0;
        let en_x_sup = pos_cursor.x >= 30.0 && pos_cursor.x <= 190.0;

        if en_y_sup && en_x_sup {
            if buttons.just_pressed(MouseButton::Left) {
                *color_volver = BackgroundColor(Color::srgb(0.85, 0.40, 0.20));
                commands.spawn(PeticionSalirChip);
                return;
            } else {
                *color_volver = BackgroundColor(Color::srgb(0.80, 0.32, 0.32));
            }
        } else {
            *color_volver = BackgroundColor(Color::srgb(0.65, 0.25, 0.25));
        }
    }

    let alto_ventana = window.height();
    let dentro_franja_y = pos_cursor.y >= (alto_ventana - 52.0) 
        && pos_cursor.y <= (alto_ventana - 10.0);

    let mut cambio_scroll = None;
    for (btn_sc, mut color_sc) in q_botones_scroll.iter_mut() {
        let dentro_x = pos_cursor.x >= btn_sc.x_min 
            && pos_cursor.x <= btn_sc.x_max;
        if dentro_franja_y && dentro_x {
            if buttons.just_pressed(MouseButton::Left) {
                cambio_scroll = Some(btn_sc.delta);
            }
            *color_sc = BackgroundColor(Color::srgb(0.28, 0.36, 0.48));
        } else {
            *color_sc = BackgroundColor(Color::srgb(0.16, 0.20, 0.28));
        }
    }

    if let Some(delta) = cambio_scroll {
        let total_chips = listar_archivos_por_ext("chips", "chp").len();
        if delta < 0 {
            scroll_barra.indice_inicio = 
                scroll_barra.indice_inicio.saturating_sub(1);
        } else if scroll_barra.indice_inicio + 1 < total_chips {
            scroll_barra.indice_inicio += 1;
        }
        reconstruir_barra_inferior_chips(
            &mut commands,
            &q_barra_inf,
            scroll_barra.indice_inicio,
        );
        return;
    }

    let centro_cam = q_camera
        .single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    let shift = keys.pressed(KeyCode::ShiftLeft) 
        || keys.pressed(KeyCode::ShiftRight);
    let ctrl = keys.pressed(KeyCode::ControlLeft) 
        || keys.pressed(KeyCode::ControlRight);

    let cantidad = if shift {
        4
    } else if ctrl {
        2
    } else {
        1
    };

    for (btn_chip, mut color_fondo) in q_botones_inf.iter_mut() {
        let dentro_x = pos_cursor.x >= btn_chip.x_min 
            && pos_cursor.x <= btn_chip.x_max;

        if dentro_franja_y && dentro_x {
            if buttons.just_pressed(MouseButton::Left) {
                *color_fondo = BackgroundColor(
                    Color::srgb(0.15, 0.65, 0.35)
                );
                if let Some(datos) = 
                    leer_datos_chip_desde_chp(&btn_chip.archivo_chp) 
                {
                    let paso = config.paso_snap_puerta;
                    let max_pines = datos
                        .pines_entrada
                        .len()
                        .max(datos.pines_salida.len())
                        .max(1);
                    let alto_est = (
                        max_pines as f32 * config.sep_pines_chip + 36.0
                    ).max(100.0);
                    let sep_vertical = (alto_est / paso).ceil() * paso;

                    for i in 0..cantidad {
                        let offset_y = -(i as f32 * sep_vertical);
                        let pos_inicial = Vec2::new(
                            centro_cam.x,
                            centro_cam.y + offset_y,
                        );
                        let (ent_chip, _, _) = crear_chip_personalizado(
                            &mut commands,
                            pos_inicial,
                            datos.clone(),
                            &config,
                        );
                        commands
                            .entity(ent_chip)
                            .insert(Arrastrando { 
                                offset: Vec2::new(0.0, offset_y) 
                            });
                    }
                }
            } else {
                *color_fondo = BackgroundColor(
                    Color::srgb(0.28, 0.46, 0.68)
                );
            }
        } else {
            *color_fondo = BackgroundColor(
                Color::srgb(0.20, 0.34, 0.50)
            );
        }
    }
}