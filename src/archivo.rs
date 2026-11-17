// ==========================================
// Fichero: src/archivo.rs
// ==========================================

// --- Bloque 1: Importaciones y Constantes de Carpetas ---
use std::fs;
use std::path::Path;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::componentes::*;
use crate::configuracion::ConfigApp;
use crate::menus::{
    reconstruir_barra_inferior_chips,
    reconstruir_barra_superior_inspeccion,
};
use crate::{
    crear_puerta,
    crear_interruptor,
    crear_led,
    crear_reloj,
    crear_display,
    crear_barras_chip_con_dimensiones,
    crear_chip_personalizado,
    crear_conector_bus,
    ancho_caja_etiqueta,
};

pub const CARPETA_CIRCUITOS: &str = "circuitos";
pub const CARPETA_CHIPS: &str = "chips";

pub fn asegurar_carpetas() {
    let _ = fs::create_dir_all(CARPETA_CIRCUITOS);
    let _ = fs::create_dir_all(CARPETA_CHIPS);
}

// ==========================================
// Fichero: src/archivo.rs
// Bloque: Bloque 1 (Función listar_archivos_por_ext)
// ==========================================
pub fn listar_archivos_por_ext(
    carpeta: &str, 
    extension: &str
) -> Vec<String> {
    asegurar_carpetas();
    let mut lista = Vec::new();
    let ext_min = extension.to_lowercase();

    if let Ok(entradas) = fs::read_dir(carpeta) {
        for entrada in entradas.flatten() {
            let path = entrada.path();
            let coincide = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase() == ext_min)
                .unwrap_or(false);

            if coincide {
                if let Some(nombre) = entrada.file_name().to_str() {
                    let ruta = format!("{}/{}", carpeta, nombre);
                    lista.push(ruta);
                }
            }
        }
    }
    lista.sort();
    lista
}
// --- Bloque 2: Peticiones Asíncronas de Guardado, Carga e Inspección ---
#[derive(Component)]
pub struct PeticionGuardar {
    pub ruta: String,
    pub es_chip: bool,
}

#[derive(Component)]
pub struct PeticionCargar(pub String);

#[derive(Component)]
pub struct PeticionEntrarChip(pub String);

#[derive(Component)]
pub struct PeticionSalirChip;

// --- Bloque 3: Creación de Ventanas Modales (Guardar, Cargar y Nombrar Pin) ---
pub fn abrir_dialogo_archivo(
    commands: &mut Commands, 
    pos_camara: Vec2,
    es_chip: bool,
) {
    let (titulo, valor_defecto) = if es_chip {
        (
            "GUARDAR CHIP EN 'chips/' (.chp) - [Enter] Guardar | [Esc] Cancelar",
            "mi_chip",
        )
    } else {
        (
            "GUARDAR CIRCUITO EN 'circuitos/' (.cir) - [Enter] Guardar | [Esc] Cancelar",
            "mi_circuito",
        )
    };

    commands.spawn((
        Sprite {
            color: Color::srgb(0.12, 0.14, 0.20),
            custom_size: Some(Vec2::new(480.0, 120.0)),
            ..default()
        },
        Transform::from_xyz(pos_camara.x, pos_camara.y, 20.0),
        DialogoArchivo {
            texto_actual: valor_defecto.to_string(),
            es_chip,
        },
    ))
    .with_children(|padre| {
        padre.spawn((
            Text2d::new(titulo),
            TextFont {
                font_size: bevy::text::FontSize::Px(12.5),
                ..default()
            },
            TextColor(Color::srgb(0.85, 0.88, 0.95)),
            Transform::from_xyz(0.0, 32.0, 0.1),
        ));

        padre.spawn((
            Sprite {
                color: Color::srgb(0.06, 0.08, 0.11),
                custom_size: Some(Vec2::new(400.0, 38.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -14.0, 0.1),
        ))
        .with_children(|caja| {
            caja.spawn((
                Text2d::new(format!("{}_", valor_defecto)),
                TextFont {
                    font_size: bevy::text::FontSize::Px(17.0),
                    ..default()
                },
                TextColor(Color::srgb(0.30, 0.95, 0.50)),
                Transform::from_xyz(0.0, 0.0, 0.2),
                TextoInputArchivo,
            ));
        });
    });
}

pub fn abrir_dialogo_etiqueta_pin(
    commands: &mut Commands,
    pos_camara: Vec2,
    nodo_objetivo: Entity,
    texto_inicial: &str,
) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.14, 0.16, 0.22),
            custom_size: Some(Vec2::new(420.0, 115.0)),
            ..default()
        },
        Transform::from_xyz(pos_camara.x, pos_camara.y, 22.0),
        DialogoEtiquetaPin {
            nodo_objetivo,
            texto_actual: texto_inicial.to_string(),
        },
    ))
    .with_children(|padre| {
        padre.spawn((
            Text2d::new(
                "NOMBRE DE CONEXIÓN / PIN (Máx 12) - [Enter] OK | [Esc] Cancelar"
            ),
            TextFont {
                font_size: bevy::text::FontSize::Px(12.0),
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.85, 0.40)),
            Transform::from_xyz(0.0, 30.0, 0.1),
        ));

        padre.spawn((
            Sprite {
                color: Color::srgb(0.06, 0.08, 0.11),
                custom_size: Some(Vec2::new(320.0, 36.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -14.0, 0.1),
        ))
        .with_children(|caja| {
            caja.spawn((
                Text2d::new(format!("{}_", texto_inicial)),
                TextFont {
                    font_size: bevy::text::FontSize::Px(17.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 0.2),
                TextoInputEtiqueta,
            ));
        });
    });
}

pub fn abrir_selector_carga(
    commands: &mut Commands, 
    pos_camara: Vec2
) {
    let archivos = listar_archivos_por_ext(CARPETA_CIRCUITOS, "cir");
    let num = archivos.len().max(1) as f32;
    let sep = 34.0;
    let alto_total = num * sep + 75.0;
    let ancho_total = 380.0;

    commands.spawn((
        Sprite {
            color: Color::srgb(0.12, 0.14, 0.20),
            custom_size: Some(Vec2::new(ancho_total, alto_total)),
            ..default()
        },
        Transform::from_xyz(pos_camara.x, pos_camara.y, 20.0),
        SelectorCarga,
    ))
    .with_children(|padre| {
        let y_titulo = alto_total / 2.0 - 24.0;
        padre.spawn((
            Text2d::new(
                "CARGAR CIRCUITO DE 'circuitos/' (.cir) - [Esc] Cerrar"
            ),
            TextFont {
                font_size: bevy::text::FontSize::Px(13.0),
                ..default()
            },
            TextColor(Color::srgb(0.85, 0.90, 1.0)),
            Transform::from_xyz(0.0, y_titulo, 0.1),
        ));

        if archivos.is_empty() {
            padre.spawn((
                Text2d::new("No hay circuitos (.cir) en 'circuitos/'"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.50, 0.50)),
                Transform::from_xyz(0.0, -10.0, 0.1),
            ));
        } else {
            let inicio_y = (archivos.len() as f32 - 1.0) * sep / 2.0 - 12.0;
            for (i, ruta_completa) in archivos.iter().enumerate() {
                let offset_y = inicio_y - (i as f32 * sep);
                let centro_btn = Vec2::new(
                    pos_camara.x, 
                    pos_camara.y + offset_y
                );

                let nombre_mostrar = Path::new(ruta_completa)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(ruta_completa.as_str());

                padre.spawn((
                    Sprite {
                        color: Color::srgb(0.22, 0.28, 0.38),
                        custom_size: Some(Vec2::new(330.0, 28.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, offset_y, 0.1),
                    BotonArchivoCarga {
                        nombre_archivo: ruta_completa.clone(),
                        centro_mundo: centro_btn,
                    },
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text2d::new(nombre_mostrar.to_string()),
                        TextFont {
                            font_size: bevy::text::FontSize::Px(14.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Transform::from_xyz(0.0, 0.0, 0.2),
                    ));
                });
            }
        }
    });
}

// --- Bloque 4: Gestión de Teclado para Nombrar Conexiones (Tecla N) ---
pub fn gestionar_nombrado_pin(
    keys: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    q_modales_otros: Query<
        Entity, 
        Or<(
            With<DialogoArchivo>, 
            With<SelectorCarga>, 
            With<MenuContextual>
        )>
    >,
    mut q_dialogo: Query<(Entity, &mut DialogoEtiquetaPin)>,
    mut q_texto_input: Query<
        &mut Text2d, 
        (With<TextoInputEtiqueta>, Without<TextoEtiquetaNodo>)
    >,
    mut q_nodos: Query<(
        Entity, 
        &Transform, 
        &NodoLogico, 
        &mut EtiquetaNodo,
        &Children
    )>,
    mut q_fondo_etq: Query<
        (&mut Sprite, &mut Transform, &mut Visibility),
        (
            With<FondoEtiquetaNodo>, 
            Without<NodoLogico>, 
            Without<TextoEtiquetaNodo>
        )
    >,
    mut q_texto_etq: Query<
        (&mut Text2d, &mut Transform), 
        (
            With<TextoEtiquetaNodo>, 
            Without<TextoInputEtiqueta>,
            Without<NodoLogico>,
            Without<FondoEtiquetaNodo>
        )
    >,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if !q_modales_otros.is_empty() {
        return;
    }

    if let Ok((ent_dlg, mut dlg)) = q_dialogo.single_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            commands.entity(ent_dlg).despawn();
            return;
        }

        if keys.just_pressed(KeyCode::Enter) {
            let nuevo_nombre = dlg.texto_actual.trim().to_string();
            if let Ok((_, _, nodo, mut etq_nodo, hijos)) = 
                q_nodos.get_mut(dlg.nodo_objetivo) 
            {
                etq_nodo.0 = nuevo_nombre.clone();
                let w_box = ancho_caja_etiqueta(&nuevo_nombre);
                let w_agarre = config.ancho_agarre_io;
                let x_nueva = if nodo.tipo == TipoPuerta::Interruptor {
                    -(w_agarre / 2.0 + 11.0) - 16.0 - w_box / 2.0
                } else {
                    (w_agarre / 2.0 + 11.0) + 16.0 + w_box / 2.0
                };

                for hijo in hijos.iter() {
                    if let Ok((mut spr_f, mut trf_f, mut vis)) = 
                        q_fondo_etq.get_mut(hijo) 
                    {
                        spr_f.custom_size = Some(Vec2::new(w_box, 18.0));
                        trf_f.translation.x = x_nueva;
                        *vis = if nuevo_nombre.is_empty() {
                            Visibility::Hidden
                        } else {
                            Visibility::Inherited
                        };
                    }
                    if let Ok((mut txt_e, mut trf_t)) = 
                        q_texto_etq.get_mut(hijo) 
                    {
                        txt_e.0 = nuevo_nombre.clone();
                        trf_t.translation.x = x_nueva;
                    }
                }
            }
            commands.entity(ent_dlg).despawn();
            return;
        }

        let mut hubo_cambio = false;
        if keys.just_pressed(KeyCode::Backspace) {
            dlg.texto_actual.pop();
            hubo_cambio = true;
        }

        let mapa_teclas: [(KeyCode, char); 37] = [
            (KeyCode::KeyA, 'A'), (KeyCode::KeyB, 'B'),
            (KeyCode::KeyC, 'C'), (KeyCode::KeyD, 'D'),
            (KeyCode::KeyE, 'E'), (KeyCode::KeyF, 'F'),
            (KeyCode::KeyG, 'G'), (KeyCode::KeyH, 'H'),
            (KeyCode::KeyI, 'I'), (KeyCode::KeyJ, 'J'),
            (KeyCode::KeyK, 'K'), (KeyCode::KeyL, 'L'),
            (KeyCode::KeyM, 'M'), (KeyCode::KeyN, 'N'),
            (KeyCode::KeyO, 'O'), (KeyCode::KeyP, 'P'),
            (KeyCode::KeyQ, 'Q'), (KeyCode::KeyR, 'R'),
            (KeyCode::KeyS, 'S'), (KeyCode::KeyT, 'T'),
            (KeyCode::KeyU, 'U'), (KeyCode::KeyV, 'V'),
            (KeyCode::KeyW, 'W'), (KeyCode::KeyX, 'X'),
            (KeyCode::KeyY, 'Y'), (KeyCode::KeyZ, 'Z'),
            (KeyCode::Digit0, '0'), (KeyCode::Digit1, '1'),
            (KeyCode::Digit2, '2'), (KeyCode::Digit3, '3'),
            (KeyCode::Digit4, '4'), (KeyCode::Digit5, '5'),
            (KeyCode::Digit6, '6'), (KeyCode::Digit7, '7'),
            (KeyCode::Digit8, '8'), (KeyCode::Digit9, '9'),
            (KeyCode::Minus, '_'),
        ];

        for (codigo, caracter) in mapa_teclas {
            if keys.just_pressed(codigo) && dlg.texto_actual.len() < 12 {
                dlg.texto_actual.push(caracter);
                hubo_cambio = true;
            }
        }

        if hubo_cambio {
            for mut txt in q_texto_input.iter_mut() {
                txt.0 = format!("{}_", dlg.texto_actual);
            }
        }
        return;
    }

    if keys.just_pressed(KeyCode::KeyN) {
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

        for (ent, trf, nodo, etq, _) in q_nodos.iter() {
            if matches!(
                nodo.tipo, 
                TipoPuerta::Interruptor | TipoPuerta::Led
            ) {
                let c = trf.translation.truncate();
                if pos_mundo.distance(c) < 38.0 {
                    abrir_dialogo_etiqueta_pin(
                        &mut commands,
                        pos_cam,
                        ent,
                        &etq.0,
                    );
                    return;
                }
            }
        }
    }
}

// --- Bloque 5: Gestión de Teclado en el Cuadro de Guardado de Archivos ---
pub fn gestionar_teclado_archivo(
    keys: Res<ButtonInput<KeyCode>>,
    mut q_dialogo: Query<(Entity, &mut DialogoArchivo)>,
    mut q_texto: Query<&mut Text2d, With<TextoInputArchivo>>,
    q_selector: Query<Entity, With<SelectorCarga>>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::Escape) {
        for ent_sel in q_selector.iter() {
            commands.entity(ent_sel).despawn();
        }
    }

    let Ok((ent_dlg, mut dialogo)) = q_dialogo.single_mut() else {
        return;
    };

    if keys.just_pressed(KeyCode::Escape) {
        commands.entity(ent_dlg).despawn();
        return;
    }

    if keys.just_pressed(KeyCode::Enter) {
        let base = dialogo.texto_actual.trim();
        if !base.is_empty() {
            let (carpeta, ext) = if dialogo.es_chip {
                (CARPETA_CHIPS, "chp")
            } else {
                (CARPETA_CIRCUITOS, "cir")
            };

            let sufijo = format!(".{}", ext);
            let nombre_limpio = base
                .strip_prefix("chips/")
                .or_else(|| base.strip_prefix("circuitos/"))
                .unwrap_or(base);

            let ruta_final = if nombre_limpio.ends_with(&sufijo) {
                format!("{}/{}", carpeta, nombre_limpio)
            } else {
                format!("{}/{}.{}", carpeta, nombre_limpio, ext)
            };

            commands.spawn(PeticionGuardar {
                ruta: ruta_final,
                es_chip: dialogo.es_chip,
            });
        }
        commands.entity(ent_dlg).despawn();
        return;
    }

    let mut hubo_cambio = false;

    if keys.just_pressed(KeyCode::Backspace) {
        dialogo.texto_actual.pop();
        hubo_cambio = true;
    }

    let mapa_teclas: [(KeyCode, char); 37] = [
        (KeyCode::KeyA, 'a'), (KeyCode::KeyB, 'b'),
        (KeyCode::KeyC, 'c'), (KeyCode::KeyD, 'd'),
        (KeyCode::KeyE, 'e'), (KeyCode::KeyF, 'f'),
        (KeyCode::KeyG, 'g'), (KeyCode::KeyH, 'h'),
        (KeyCode::KeyI, 'i'), (KeyCode::KeyJ, 'j'),
        (KeyCode::KeyK, 'k'), (KeyCode::KeyL, 'l'),
        (KeyCode::KeyM, 'm'), (KeyCode::KeyN, 'n'),
        (KeyCode::KeyO, 'o'), (KeyCode::KeyP, 'p'),
        (KeyCode::KeyQ, 'q'), (KeyCode::KeyR, 'r'),
        (KeyCode::KeyS, 's'), (KeyCode::KeyT, 't'),
        (KeyCode::KeyU, 'u'), (KeyCode::KeyV, 'v'),
        (KeyCode::KeyW, 'w'), (KeyCode::KeyX, 'x'),
        (KeyCode::KeyY, 'y'), (KeyCode::KeyZ, 'z'),
        (KeyCode::Digit0, '0'), (KeyCode::Digit1, '1'),
        (KeyCode::Digit2, '2'), (KeyCode::Digit3, '3'),
        (KeyCode::Digit4, '4'), (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'), (KeyCode::Digit7, '7'),
        (KeyCode::Digit8, '8'), (KeyCode::Digit9, '9'),
        (KeyCode::Minus, '_'),
    ];

    for (codigo, caracter) in mapa_teclas {
        if keys.just_pressed(codigo) && dialogo.texto_actual.len() < 24 {
            dialogo.texto_actual.push(caracter);
            hubo_cambio = true;
        }
    }

    if hubo_cambio {
        for mut txt in q_texto.iter_mut() {
            txt.0 = format!("{}_", dialogo.texto_actual);
        }
    }
}

// --- Bloque 6: Gestión de Clics en el Selector de Carga (.cir) ---
pub fn gestionar_clic_selector_carga(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    q_selector: Query<Entity, With<SelectorCarga>>,
    mut q_botones: Query<(&BotonArchivoCarga, &mut Sprite)>,
    mut commands: Commands,
) {
    let Ok(ent_sel) = q_selector.single() else {
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

    let mut archivo_elegido = None;

    for (btn, mut sprite) in q_botones.iter_mut() {
        let dentro_x = (pos_mundo.x - btn.centro_mundo.x).abs() < 165.0;
        let dentro_y = (pos_mundo.y - btn.centro_mundo.y).abs() < 14.0;

        if dentro_x && dentro_y {
            sprite.color = Color::srgb(0.35, 0.48, 0.65);
            if buttons.just_pressed(MouseButton::Left) {
                archivo_elegido = Some(btn.nombre_archivo.clone());
            }
        } else {
            sprite.color = Color::srgb(0.22, 0.28, 0.38);
        }
    }

    if buttons.just_pressed(MouseButton::Left) {
        if let Some(nombre) = archivo_elegido {
            commands.spawn(PeticionCargar(nombre));
        }
        commands.entity(ent_sel).despawn();
    }
}

// ==========================================
// Fichero: src/archivo.rs
// Bloque: Bloque 7: Lector de Chips (.chp) con Soporte de Sub-Chips y ConectorBus
// ==========================================
pub fn leer_datos_chip_desde_chp(
    ruta_archivo: &str
) -> Option<DatosChip> {
    let contenido = fs::read_to_string(ruta_archivo).ok()?;

    let nombre_limpio = Path::new(ruta_archivo)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("CHIP")
        .to_uppercase();

    let mut nodos = Vec::new();
    let mut cables = Vec::new();
    let mut cables_bus = Vec::new();
    let mut entradas_con_y: Vec<(usize, f32, String)> = Vec::new();
    let mut salidas_con_y: Vec<(usize, f32, String)> = Vec::new();

    for linea in contenido.lines() {
        let partes: Vec<&str> = linea.split('|').collect();
        if partes.is_empty() {
            continue;
        }

        if partes[0] == "NODO" && partes.len() >= 6 {
            let idx = nodos.len();
            let cod = partes[1].to_string();
            let x: f32 = partes[2].parse().unwrap_or(0.0);
            let y: f32 = partes[3].parse().unwrap_or(0.0);
            let tag_barra = if partes.len() >= 7 {
                partes[6].trim()
            } else {
                ""
            };
            let etiqueta_pin = if partes.len() >= 8 {
                partes[7].trim().to_string()
            } else {
                String::new()
            };

            let sub_chip = if let Some(nom_sub) = 
                cod.strip_prefix("CHIP:") 
            {
                let arch_sub_min = format!(
                    "{}/{}.chp", 
                    CARPETA_CHIPS, 
                    nom_sub.to_lowercase()
                );
                let arch_sub_orig = format!(
                    "{}/{}.chp", 
                    CARPETA_CHIPS, 
                    nom_sub
                );
                leer_datos_chip_desde_chp(&arch_sub_min)
                    .or_else(|| {
                        leer_datos_chip_desde_chp(&arch_sub_orig)
                    })
                    .map(Box::new)
            } else {
                None
            };

            nodos.push(NodoInternoChip {
                cod_tipo: cod.clone(),
                sub_chip,
            });

            let es_pin_in = tag_barra == "PIN_IN"
                || (tag_barra != "PIN_OUT" && (x - (-500.0)).abs() < 25.0);
            let es_pin_out = tag_barra == "PIN_OUT"
                || (tag_barra != "PIN_IN" && (x - 500.0).abs() < 25.0);

            if cod == "INT" && es_pin_in {
                entradas_con_y.push((idx, y, etiqueta_pin));
            } else if cod == "LED" && es_pin_out {
                salidas_con_y.push((idx, y, etiqueta_pin));
            }
        } else if partes[0] == "CABLE" && partes.len() >= 4 {
            let orig: usize = partes[1].parse().unwrap_or(9999);
            let dest: usize = partes[2].parse().unwrap_or(9999);
            let pin_in: usize = partes[3].parse().unwrap_or(0);
            let pin_out: usize = if partes.len() >= 6 {
                partes[5].trim().parse().unwrap_or(0)
            } else {
                0
            };
            cables.push((orig, pin_out, dest, pin_in));
        } else if partes[0] == "CABLE_BUS" && partes.len() >= 3 {
            let orig: usize = partes[1].parse().unwrap_or(9999);
            let dest: usize = partes[2].parse().unwrap_or(9999);
            cables_bus.push((orig, dest));
        }
    }

    entradas_con_y.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
    });
    salidas_con_y.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
    });

    let pines_entrada = entradas_con_y
        .iter()
        .map(|(i, _, _)| *i)
        .collect();
    let nombres_entrada = entradas_con_y
        .into_iter()
        .map(|(_, _, n)| n)
        .collect();

    let pines_salida = salidas_con_y
        .iter()
        .map(|(i, _, _)| *i)
        .collect();
    let nombres_salida = salidas_con_y
        .into_iter()
        .map(|(_, _, n)| n)
        .collect();

    Some(DatosChip {
        nombre: nombre_limpio,
        nodos,
        cables,
        cables_bus,
        pines_entrada,
        nombres_entrada,
        pines_salida,
        nombres_salida,
    })
}

// ==========================================
// Fichero: src/archivo.rs
// Bloque: Bloque 8: Procesador de Guardado, Carga, Inspección y Deshacer (Ctrl + Z)
// ==========================================
pub fn procesar_guardado_y_carga(
    keys: Res<ButtonInput<KeyCode>>,
    q_guardar: Query<(Entity, &PeticionGuardar)>,
    q_cargar: Query<(Entity, &PeticionCargar)>,
    q_entrar_chip: Query<(Entity, &PeticionEntrarChip)>,
    q_salir_chip: Query<Entity, With<PeticionSalirChip>>,
    q_ui_barras: (
        Query<Entity, With<BarraInferiorChips>>,
        Query<Entity, With<BarraSuperiorInspeccion>>,
        Query<Entity, With<BarraChip>>,
    ),
    q_nodos: Query<(
        Entity, 
        &Transform, 
        &NodoLogico, 
        &Children, 
        Option<&PegadoABarra>,
        Option<&InstanciaChip>,
        Option<&DatosConectorBus>,
        Option<&EtiquetaNodo>,
    )>,
    q_cables: Query<(Entity, &Cable)>,
    q_cables_bus: Query<(Entity, &CableBus)>,
    q_componentes_internos: (
        Query<&BotonInterruptor>,
        Query<&ControlReloj>,
        Query<&TipoConector>,
        Query<&PinEntrada>,
        Query<&PinSalida>,
    ),
    mut pila_inspeccion: ResMut<PilaInspeccion>,
    mut historial_undo: ResMut<HistorialUndo>,
    scroll_barra: Res<DesplazamientoBarraChips>,
    mut recursos_escena: (
        Commands,
        ResMut<Assets<Mesh>>,
        ResMut<Assets<ColorMaterial>>,
    ),
    config: Res<ConfigApp>,
) {
    let (
        ref q_barra_inf, 
        ref q_barra_sup, 
        ref q_barras
    ) = q_ui_barras;
    let (
        ref q_botones,
        ref q_relojes,
        ref q_conectores,
        ref q_pines_in,
        ref q_pines_out,
    ) = q_componentes_internos;
    let (
        ref mut commands, 
        ref mut meshes, 
        ref mut materials
    ) = recursos_escena;

    let ctrl_presionado = keys.pressed(KeyCode::ControlLeft) 
        || keys.pressed(KeyCode::ControlRight);

    if ctrl_presionado && keys.just_pressed(KeyCode::KeyZ) {
        if historial_undo.estados.len() > 1 {
            historial_undo.estados.pop();
            if let Some(estado_previo) = historial_undo.estados.last().cloned() {
                reconstruir_circuito_desde_texto(
                    &estado_previo,
                    &q_nodos,
                    &q_cables,
                    &q_cables_bus,
                    q_barras,
                    commands,
                    meshes,
                    materials,
                    &config,
                );
            }
        }
        return;
    }

    let instantanea_actual = serializar_circuito_a_texto(
        &q_nodos,
        &q_cables,
        &q_cables_bus,
        q_botones,
        q_relojes,
        q_conectores,
        q_pines_in,
        q_pines_out,
    );

    if historial_undo
        .estados
        .last()
        .map(|s| s != &instantanea_actual)
        .unwrap_or(true)
    {
        historial_undo.estados.push(instantanea_actual);
        if historial_undo.estados.len() > 40 {
            historial_undo.estados.remove(0);
        }
    }

    for (ent_pet, peticion) in q_guardar.iter() {
        guardar_en_disco(
            &peticion.ruta,
            &q_nodos,
            &q_cables,
            &q_cables_bus,
            q_botones,
            q_relojes,
            q_conectores,
            q_pines_in,
            q_pines_out,
        );
        if peticion.es_chip {
            reconstruir_barra_inferior_chips(
                commands, 
                q_barra_inf, 
                scroll_barra.indice_inicio
            );
        }
        commands.entity(ent_pet).despawn();
    }

    for (ent_pet, peticion) in q_cargar.iter() {
        pila_inspeccion.niveles.clear();
        historial_undo.estados.clear();
        reconstruir_barra_superior_inspeccion(
            commands, 
            q_barra_sup, 
            &pila_inspeccion
        );
        cargar_desde_disco(
            &peticion.0,
            &q_nodos,
            &q_cables,
            &q_cables_bus,
            q_barras,
            commands,
            meshes,
            materials,
            &config,
        );
        commands.entity(ent_pet).despawn();
    }

    for (ent_pet, pet_entrar) in q_entrar_chip.iter() {
        let estado_padre = serializar_circuito_a_texto(
            &q_nodos,
            &q_cables,
            &q_cables_bus,
            q_botones,
            q_relojes,
            q_conectores,
            q_pines_in,
            q_pines_out,
        );
        pila_inspeccion.niveles.push((
            pet_entrar.0.clone(), 
            estado_padre
        ));
        historial_undo.estados.clear();

        reconstruir_barra_superior_inspeccion(
            commands, 
            q_barra_sup, 
            &pila_inspeccion
        );
        cargar_desde_disco(
            &pet_entrar.0,
            &q_nodos,
            &q_cables,
            &q_cables_bus,
            q_barras,
            commands,
            meshes,
            materials,
            &config,
        );
        commands.entity(ent_pet).despawn();
    }

    for ent_pet in q_salir_chip.iter() {
        if let Some((_, estado_padre)) = pila_inspeccion.niveles.pop() {
            historial_undo.estados.clear();
            reconstruir_barra_superior_inspeccion(
                commands, 
                q_barra_sup, 
                &pila_inspeccion
            );
            reconstruir_circuito_desde_texto(
                &estado_padre,
                &q_nodos,
                &q_cables,
                &q_cables_bus,
                q_barras,
                commands,
                meshes,
                materials,
                &config,
            );
        }
        commands.entity(ent_pet).despawn();
    }
}

// ==========================================
// Fichero: src/archivo.rs
// Bloque: Bloque 9: Lógica de Serialización a Texto
// ==========================================
pub fn serializar_circuito_a_texto(
    q_nodos: &Query<(
        Entity, 
        &Transform, 
        &NodoLogico, 
        &Children, 
        Option<&PegadoABarra>,
        Option<&InstanciaChip>,
        Option<&DatosConectorBus>,
        Option<&EtiquetaNodo>,
    )>,
    q_cables: &Query<(Entity, &Cable)>,
    q_cables_bus: &Query<(Entity, &CableBus)>,
    q_botones: &Query<&BotonInterruptor>,
    q_relojes: &Query<&ControlReloj>,
    q_conectores: &Query<&TipoConector>,
    q_pines_in: &Query<&PinEntrada>,
    q_pines_out: &Query<&PinSalida>,
) -> String {
    let mut lineas_nodos = Vec::new();
    let mut lista_nodos = Vec::new();

    let mut nodos_ordenados: Vec<_> = q_nodos.iter().collect();
    nodos_ordenados.sort_by_key(|(ent, _, _, _, _, _, _, _)| *ent);

    for (ent_nodo, trf, nodo, hijos, opt_pegado, opt_chip, opt_bus, opt_etq) in nodos_ordenados {
        let mut estado_bool = 0;
        let mut indice_frec = 1;
        let mut salidas_nodo = Vec::new();
        let mut entradas_nodo = Vec::new();

        for hijo in hijos.iter() {
            if let Ok(btn) = q_botones.get(hijo) {
                if btn.estado {
                    estado_bool = 1;
                }
            }
            if let Ok(rel) = q_relojes.get(hijo) {
                indice_frec = rel.indice_frec;
            }
            if let Ok(tipo_con) = q_conectores.get(hijo) {
                match tipo_con {
                    TipoConector::Salida => {
                        let pin_s = q_pines_out
                            .get(hijo)
                            .map(|p| p.0)
                            .unwrap_or(0);
                        salidas_nodo.push((pin_s, hijo));
                    }
                    TipoConector::Entrada => {
                        let pin_e = q_pines_in
                            .get(hijo)
                            .map(|p| p.0)
                            .unwrap_or(0);
                        entradas_nodo.push((pin_e, hijo));
                    }
                }
            }
        }

        let cod_tipo = match nodo.tipo {
            TipoPuerta::And => "AND".to_string(),
            TipoPuerta::Or => "OR".to_string(),
            TipoPuerta::Not => "NOT".to_string(),
            TipoPuerta::Nand => "NAND".to_string(),
            TipoPuerta::Nor => "NOR".to_string(),
            TipoPuerta::Xor => "XOR".to_string(),
            TipoPuerta::Xnor => "XNOR".to_string(),
            TipoPuerta::TriState => "TRI".to_string(),
            TipoPuerta::Interruptor => "INT".to_string(),
            TipoPuerta::Led => "LED".to_string(),
            TipoPuerta::Reloj => "CLK".to_string(),
            TipoPuerta::Display7Seg => "D7S".to_string(),
            TipoPuerta::Display4Bits => "D4B".to_string(),
            TipoPuerta::ConectorBus => {
                let db = opt_bus.copied().unwrap_or(DatosConectorBus {
                    num_hilos: 8,
                    rotacion: 0,
                    modo: ModoDireccionBus::HaciaBus,
                });
                format!(
                    "BUS:{}:{}:{}", 
                    db.num_hilos, 
                    db.rotacion, 
                    db.modo.a_u8()
                )
            }
            TipoPuerta::Chip => {
                let nom = opt_chip
                    .map(|c| c.datos.nombre.clone())
                    .unwrap_or_else(|| "CHIP".to_string());
                format!("CHIP:{}", nom)
            }
        };

        let x = trf.translation.x;
        let y = trf.translation.y;

        let tag_barra = match opt_pegado {
            Some(PegadoABarra(BarraChip::IzquierdaEntradas)) => "PIN_IN",
            Some(PegadoABarra(BarraChip::DerechaSalidas)) => "PIN_OUT",
            None => {
                if cod_tipo == "INT" && (x - (-500.0)).abs() < 20.0 {
                    "PIN_IN"
                } else if cod_tipo == "LED" && (x - 500.0).abs() < 20.0 {
                    "PIN_OUT"
                } else {
                    "LIBRE"
                }
            }
        };

        let texto_etq = opt_etq.map(|e| e.0.as_str()).unwrap_or("");

        let linea = format!(
            "NODO|{}|{}|{}|{}|{}|{}|{}",
            cod_tipo, x, y, estado_bool, indice_frec, tag_barra, texto_etq
        );
        lineas_nodos.push(linea);

        lista_nodos.push((ent_nodo, salidas_nodo, entradas_nodo));
    }

    let mut lineas_cables = Vec::new();
    let mut cables_ordenados: Vec<_> = q_cables.iter().collect();
    cables_ordenados.sort_by_key(|(ent, _)| *ent);

    for (_, cable) in cables_ordenados {
        let mut idx_origen = None;
        let mut pin_origen = 0;
        let mut idx_destino = None;
        let mut pin_destino = 0;

        for (i, (_, salidas, entradas)) in lista_nodos.iter().enumerate() {
            for (p_out, ent_out) in salidas.iter() {
                if *ent_out == cable.origen {
                    idx_origen = Some(i);
                    pin_origen = *p_out;
                }
            }
            for (p_in, ent_in) in entradas.iter() {
                if *ent_in == cable.destino {
                    idx_destino = Some(i);
                    pin_destino = *p_in;
                }
            }
        }

        if let (Some(orig), Some(dest)) = (idx_origen, idx_destino) {
            let codos_str: Vec<String> = cable
                .puntos_intermedios
                .iter()
                .map(|p| format!("{},{}", p.x, p.y))
                .collect();

            let linea = format!(
                "CABLE|{}|{}|{}|{}|{}",
                orig,
                dest,
                pin_destino,
                codos_str.join(";"),
                pin_origen
            );
            lineas_cables.push(linea);
        }
    }

    let mut cables_bus_ordenados: Vec<_> = q_cables_bus.iter().collect();
    cables_bus_ordenados.sort_by_key(|(ent, _)| *ent);

    for (_, cb) in cables_bus_ordenados {
        let idx_orig = lista_nodos
            .iter()
            .position(|(e, _, _)| *e == cb.origen_nodo);
        let idx_dest = lista_nodos
            .iter()
            .position(|(e, _, _)| *e == cb.destino_nodo);

        if let (Some(o), Some(d)) = (idx_orig, idx_dest) {
            let codos_str: Vec<String> = cb
                .puntos_intermedios
                .iter()
                .map(|p| format!("{},{}", p.x, p.y))
                .collect();

            let linea_cb = format!(
                "CABLE_BUS|{}|{}|{}|{}",
                o,
                d,
                codos_str.join(";"),
                cb.num_hilos
            );
            lineas_cables.push(linea_cb);
        }
    }

    lineas_nodos.extend(lineas_cables);
    lineas_nodos.join("\n")
}

fn guardar_en_disco(
    ruta_archivo: &str,
    q_nodos: &Query<(
        Entity, 
        &Transform, 
        &NodoLogico, 
        &Children, 
        Option<&PegadoABarra>,
        Option<&InstanciaChip>,
        Option<&DatosConectorBus>,
        Option<&EtiquetaNodo>,
    )>,
    q_cables: &Query<(Entity, &Cable)>,
    q_cables_bus: &Query<(Entity, &CableBus)>,
    q_botones: &Query<&BotonInterruptor>,
    q_relojes: &Query<&ControlReloj>,
    q_conectores: &Query<&TipoConector>,
    q_pines_in: &Query<&PinEntrada>,
    q_pines_out: &Query<&PinSalida>,
) {
    asegurar_carpetas();
    let contenido = serializar_circuito_a_texto(
        q_nodos,
        q_cables,
        q_cables_bus,
        q_botones,
        q_relojes,
        q_conectores,
        q_pines_in,
        q_pines_out,
    );

    if let Err(err) = fs::write(ruta_archivo, contenido) {
        println!("Error al guardar {}: {}", ruta_archivo, err);
    } else {
        println!("Guardado correctamente en {}", ruta_archivo);
    }
}

// ==========================================
// Fichero: src/archivo.rs
// Bloque: Bloque 10: Lógica de Lectura y Reconstrucción en Lienzo
// ==========================================
pub fn reconstruir_circuito_desde_texto(
    contenido: &str,
    q_nodos: &Query<(
        Entity, 
        &Transform, 
        &NodoLogico, 
        &Children, 
        Option<&PegadoABarra>,
        Option<&InstanciaChip>,
        Option<&DatosConectorBus>,
        Option<&EtiquetaNodo>,
    )>,
    q_cables: &Query<(Entity, &Cable)>,
    q_cables_bus: &Query<(Entity, &CableBus)>,
    q_barras: &Query<Entity, With<BarraChip>>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    config: &ConfigApp,
) {
    for (ent_nodo, _, _, _, _, _, _, _) in q_nodos.iter() {
        commands.entity(ent_nodo).despawn();
    }
    for (ent_cable, _) in q_cables.iter() {
        commands.entity(ent_cable).despawn();
    }
    for (ent_cb, _) in q_cables_bus.iter() {
        commands.entity(ent_cb).despawn();
    }
    for ent_b in q_barras.iter() {
        commands.entity(ent_b).despawn();
    }

    let mut necesita_barras = false;
    let mut x_barra_izq: Option<f32> = None;
    let mut x_barra_der: Option<f32> = None;
    let mut max_abs_y: f32 = 400.0;
    let mut entidades_nodos: Vec<Entity> = Vec::new();
    let mut conectores_creados: Vec<(Vec<Entity>, Vec<Entity>)> = Vec::new();

    for linea in contenido.lines() {
        let partes: Vec<&str> = linea.split('|').collect();
        if partes.is_empty() {
            continue;
        }

        if partes[0] == "NODO" && partes.len() >= 6 {
            let cod = partes[1];
            let x: f32 = partes[2].parse().unwrap_or(0.0);
            let y: f32 = partes[3].parse().unwrap_or(0.0);
            let est: bool = partes[4] == "1";
            let frec: usize = partes[5].parse().unwrap_or(1);
            let tag_barra = if partes.len() >= 7 {
                partes[6]
            } else {
                "LIBRE"
            };
            let etq_nodo = if partes.len() >= 8 {
                partes[7]
            } else {
                ""
            };

            if tag_barra == "PIN_IN" {
                necesita_barras = true;
                x_barra_izq = Some(x);
                max_abs_y = max_abs_y.max(y.abs() + 80.0);
            } else if tag_barra == "PIN_OUT" {
                necesita_barras = true;
                x_barra_der = Some(x);
                max_abs_y = max_abs_y.max(y.abs() + 80.0);
            }

            let pos = Vec2::new(x, y);

            let (ent_creada, salidas_vec, entradas_vec) = if let Some(nom_chip) = 
                cod.strip_prefix("CHIP:") 
            {
                let arch_sub_min = format!(
                    "{}/{}.chp", 
                    CARPETA_CHIPS, 
                    nom_chip.to_lowercase()
                );
                let arch_sub_orig = format!(
                    "{}/{}.chp", 
                    CARPETA_CHIPS, 
                    nom_chip
                );
                if let Some(datos_sub) = leer_datos_chip_desde_chp(&arch_sub_min)
                    .or_else(|| leer_datos_chip_desde_chp(&arch_sub_orig)) 
                {
                    crear_chip_personalizado(
                        commands, 
                        pos, 
                        datos_sub, 
                        config
                    )
                } else {
                    (Entity::PLACEHOLDER, Vec::new(), Vec::new())
                }
            } else if let Some(cfg_bus) = cod.strip_prefix("BUS:") {
                let p_b: Vec<&str> = cfg_bus.split(':').collect();
                let num_hilos: usize = p_b
                    .first()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(8);
                let rotacion: u8 = p_b
                    .get(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let modo_u8: u8 = p_b
                    .get(2)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);

                crear_conector_bus(
                    commands,
                    pos,
                    DatosConectorBus {
                        num_hilos,
                        rotacion,
                        modo: ModoDireccionBus::desde_u8(modo_u8),
                    },
                    config,
                )
            } else if cod == "INT" {
                let (ent_i, s_i, e_i) = crear_interruptor(
                    commands, meshes, materials, pos, est, etq_nodo, config
                );
                if tag_barra == "PIN_IN" {
                    commands
                        .entity(ent_i)
                        .insert(PegadoABarra(BarraChip::IzquierdaEntradas));
                }
                (ent_i, s_i.into_iter().collect(), e_i)
            } else if cod == "LED" {
                let (ent_l, s_l, e_l) = crear_led(
                    commands, meshes, materials, pos, etq_nodo, config
                );
                if tag_barra == "PIN_OUT" {
                    commands
                        .entity(ent_l)
                        .insert(PegadoABarra(BarraChip::DerechaSalidas));
                }
                (ent_l, s_l.into_iter().collect(), e_l)
            } else {
                let info_simple = match cod {
                    "AND" => crear_puerta(
                        commands, meshes, materials, pos, TipoPuerta::And, config
                    ),
                    "OR" => crear_puerta(
                        commands, meshes, materials, pos, TipoPuerta::Or, config
                    ),
                    "NOT" => crear_puerta(
                        commands, meshes, materials, pos, TipoPuerta::Not, config
                    ),
                    "NAND" => crear_puerta(
                        commands, meshes, materials, pos, TipoPuerta::Nand, config
                    ),
                    "NOR" => crear_puerta(
                        commands, meshes, materials, pos, TipoPuerta::Nor, config
                    ),
                    "XOR" => crear_puerta(
                        commands, meshes, materials, pos, TipoPuerta::Xor, config
                    ),
                    "XNOR" => crear_puerta(
                        commands, meshes, materials, pos, TipoPuerta::Xnor, config
                    ),
                    "TRI" => crear_puerta(
                        commands, meshes, materials, pos, TipoPuerta::TriState, config
                    ),
                    "CLK" => crear_reloj(
                        commands, meshes, materials, pos, frec, config
                    ),
                    "D7S" => crear_display(commands, pos, false, config),
                    "D4B" => crear_display(commands, pos, true, config),
                    _ => (None, Vec::new()),
                };
                let v_sal = info_simple.0.into_iter().collect();
                (Entity::PLACEHOLDER, v_sal, info_simple.1)
            };

            entidades_nodos.push(ent_creada);
            conectores_creados.push((salidas_vec, entradas_vec));
        } else if partes[0] == "CABLE" && partes.len() >= 4 {
            let orig_idx: usize = partes[1].parse().unwrap_or(9999);
            let dest_idx: usize = partes[2].parse().unwrap_or(9999);
            let pin_in_idx: usize = partes[3].parse().unwrap_or(0);
            let pin_out_idx: usize = if partes.len() >= 6 {
                partes[5].parse().unwrap_or(0)
            } else {
                0
            };

            let mut puntos_intermedios = Vec::new();
            if partes.len() >= 5 && !partes[4].is_empty() {
                for par in partes[4].split(';') {
                    let coords: Vec<&str> = par.split(',').collect();
                    if coords.len() == 2 {
                        let cx: f32 = coords[0].parse().unwrap_or(0.0);
                        let cy: f32 = coords[1].parse().unwrap_or(0.0);
                        puntos_intermedios.push(Vec2::new(cx, cy));
                    }
                }
            }

            if orig_idx < conectores_creados.len() 
                && dest_idx < conectores_creados.len() 
            {
                let salidas_orig = &conectores_creados[orig_idx].0;
                let entradas_dest = &conectores_creados[dest_idx].1;

                if let (Some(&origen), Some(&destino)) = (
                    salidas_orig.get(pin_out_idx),
                    entradas_dest.get(pin_in_idx),
                ) {
                    commands.spawn(Cable { 
                        origen, 
                        destino,
                        puntos_intermedios,
                    });
                }
            }
        } else if partes[0] == "CABLE_BUS" && partes.len() >= 5 {
            let orig_idx: usize = partes[1].parse().unwrap_or(9999);
            let dest_idx: usize = partes[2].parse().unwrap_or(9999);
            let num_hilos: usize = partes[4].parse().unwrap_or(8);

            let mut puntos_intermedios = Vec::new();
            if !partes[3].is_empty() {
                for par in partes[3].split(';') {
                    let coords: Vec<&str> = par.split(',').collect();
                    if coords.len() == 2 {
                        let cx: f32 = coords[0].parse().unwrap_or(0.0);
                        let cy: f32 = coords[1].parse().unwrap_or(0.0);
                        puntos_intermedios.push(Vec2::new(cx, cy));
                    }
                }
            }

            if let (Some(&ent_o), Some(&ent_d)) = (
                entidades_nodos.get(orig_idx),
                entidades_nodos.get(dest_idx),
            ) {
                if ent_o != Entity::PLACEHOLDER && ent_d != Entity::PLACEHOLDER {
                    commands.spawn(CableBus {
                        origen_nodo: ent_o,
                        destino_nodo: ent_d,
                        puntos_intermedios,
                        num_hilos,
                        estados: vec![false; num_hilos.max(2)],
                    });
                }
            }
        }
    }

    if necesita_barras {
        let x_izq = x_barra_izq.unwrap_or(-500.0);
        let x_der = x_barra_der.unwrap_or(500.0);
        let alto_barras = (max_abs_y * 2.0).max(800.0);
        crear_barras_chip_con_dimensiones(
            commands, 
            config, 
            x_izq, 
            x_der, 
            alto_barras
        );
    }
}

fn cargar_desde_disco(
    ruta_archivo: &str,
    q_nodos: &Query<(
        Entity, 
        &Transform, 
        &NodoLogico, 
        &Children, 
        Option<&PegadoABarra>,
        Option<&InstanciaChip>,
        Option<&DatosConectorBus>,
        Option<&EtiquetaNodo>,
    )>,
    q_cables: &Query<(Entity, &Cable)>,
    q_cables_bus: &Query<(Entity, &CableBus)>,
    q_barras: &Query<Entity, With<BarraChip>>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    config: &ConfigApp,
) {
    let Ok(contenido) = fs::read_to_string(ruta_archivo) else {
        println!("No se encontró el fichero {}", ruta_archivo);
        return;
    };

    reconstruir_circuito_desde_texto(
        &contenido,
        q_nodos,
        q_cables,
        q_cables_bus,
        q_barras,
        commands,
        meshes,
        materials,
        config,
    );

    println!("Circuito cargado correctamente desde {}", ruta_archivo);
}