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

// --- Bloque 2: Gestión de Hover y Clics en el Menú Contextual ---
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
                let paso = if matches!(
                    tipo, 
                    TipoPuerta::Interruptor | TipoPuerta::Led
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
            }
        }
    }

    commands.entity(ent_menu).despawn();
    for (ent_sub, _) in q_submenu.iter() {
        commands.entity(ent_sub).despawn();
    }
}

// --- Bloque 3: Constructores del Menú Contextual y Submenús ---
pub fn abrir_menu_contextual(
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

pub fn abrir_submenu(
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

// --- Bloque 4: Constructores de las Barras Flotantes UI (Inferior y Superior) ---
pub fn reconstruir_barra_inferior_chips(
    commands: &mut Commands,
    q_barra_existente: &Query<Entity, With<BarraInferiorChips>>,
) {
    for ent in q_barra_existente.iter() {
        commands.entity(ent).despawn();
    }

    let archivos_chp = listar_archivos_por_ext("chp");
    let mut chips_validos = Vec::new();

    for arch in archivos_chp {
        if let Some(datos) = leer_datos_chip_desde_chp(&arch) {
            chips_validos.push((arch, datos.nombre));
        }
    }

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(16.0),
            right: Val::Px(16.0),
            height: Val::Px(46.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            column_gap: Val::Px(10.0),
            padding: UiRect::horizontal(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.09, 0.12, 0.92)),
        BarraInferiorChips,
    ))
    .with_children(|barra| {
        barra.spawn((
            Node {
                width: Val::Px(60.0),
                ..default()
            },
            Text::new("CHIPS:"),
            TextFont {
                font_size: bevy::text::FontSize::Px(14.0),
                ..default()
            },
            TextColor(Color::srgb(0.65, 0.75, 0.90)),
        ));

        if chips_validos.is_empty() {
            barra.spawn((
                Text::new(
                    "(Usa Chip >> Guardar .chp con entradas/salidas en las barras para crear un Chip)"
                ),
                TextFont {
                    font_size: bevy::text::FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.52, 0.58)),
            ));
        } else {
            let mut cursor_x = 100.0;
            let ancho_fijo_btn = 110.0;
            let gap = 10.0;

            for (archivo_chp, nombre_chip) in chips_validos {
                let x_min = cursor_x;
                let x_max = cursor_x + ancho_fijo_btn;
                cursor_x = x_max + gap;

                barra.spawn((
                    Node {
                        width: Val::Px(ancho_fijo_btn),
                        height: Val::Px(32.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.20, 0.34, 0.50)),
                    BotonChipInferior { 
                        archivo_chp,
                        x_min,
                        x_max,
                    },
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(format!("+ {}", nombre_chip)),
                        TextFont {
                            font_size: bevy::text::FontSize::Px(14.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        }
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

    let ruta_migas: Vec<&str> = pila
        .niveles
        .iter()
        .map(|(nom, _)| nom.as_str())
        .collect();
    let texto_ruta = format!(
        "VIENDO INTERIOR DE: Principal > {}",
        ruta_migas.join(" > ")
    );

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(16.0),
            right: Val::Px(16.0),
            height: Val::Px(44.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            column_gap: Val::Px(16.0),
            padding: UiRect::horizontal(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.14, 0.10, 0.22, 0.94)),
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
                Text::new("< VOLVER ATRÁS"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        barra.spawn((
            Text::new(texto_ruta),
            TextFont {
                font_size: bevy::text::FontSize::Px(14.0),
                ..default()
            },
            TextColor(Color::srgb(0.90, 0.85, 1.0)),
        ));
    });
}

// --- Bloque 5: Interacción con las Barras Flotantes UI ---
pub fn gestionar_clic_barras_ui(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<&Transform, With<CamaraPrincipal>>,
    mut q_botones_inf: Query<
        (&BotonChipInferior, &mut BackgroundColor),
        Without<BotonVolverInspeccion>,
    >,
    mut q_boton_volver: Query<
        &mut BackgroundColor,
        With<BotonVolverInspeccion>,
    >,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    let Ok(window) = q_windows.single() else { return; };
    let Some(pos_cursor) = window.cursor_position() else {
        return;
    };

    // 1. Clic en el botón superior [< VOLVER ATRÁS]
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

    // 2. Clic en los botones de la barra inferior de Chips
    let alto_ventana = window.height();
    let dentro_franja_y = pos_cursor.y >= (alto_ventana - 56.0) 
        && pos_cursor.y <= (alto_ventana - 10.0);

    let centro_cam = q_camera
        .single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

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
                    let pos_snap = Vec2::new(
                        (centro_cam.x / paso).round() * paso,
                        (centro_cam.y / paso).round() * paso,
                    );
                    crear_chip_personalizado(
                        &mut commands,
                        pos_snap,
                        datos,
                        &config,
                    );
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