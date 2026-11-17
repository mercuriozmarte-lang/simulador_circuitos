// ==========================================
// Fichero: src/archivo.rs
// Bloque: Bloque 1: Importaciones y Constantes de Carpetas
// ==========================================
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
    ancho_caja_etiqueta,
};

pub const CARPETA_CIRCUITOS: &str = "circuitos";
pub const CARPETA_CHIPS: &str = "chips";

pub fn asegurar_carpetas() {
    let _ = fs::create_dir_all(CARPETA_CIRCUITOS);
    let _ = fs::create_dir_all(CARPETA_CHIPS);
}

// --- Bloque 2: Eventos de Guardado, Carga e Inspección de Chips ---
#[derive(Component)]
pub struct PeticionGuardar(pub String);

#[derive(Component)]
pub struct PeticionCargar(pub String);

#[derive(Component)]
pub struct PeticionEntrarChip(pub String);

#[derive(Component)]
pub struct PeticionSalirChip;

// --- Bloque 3: Creador del Cuadro de Diálogo (.cir en circuitos/ o .chp en chips/) ---
pub fn abrir_dialogo_archivo(
    commands: &mut Commands,
    pos_cam: Vec2,
    es_chip: bool,
) {
    asegurar_carpetas();

    let (titulo, nombre_defecto, carpeta, ext) = if es_chip {
        (
            "GUARDAR CHIP EN chips/ (Enter = OK, Esc = Cancelar)",
            "mi_chip".to_string(),
            CARPETA_CHIPS,
            ".chp",
        )
    } else {
        (
            "GUARDAR CIRCUITO EN circuitos/ (Enter = OK, Esc = Cancelar)",
            "circuito".to_string(),
            CARPETA_CIRCUITOS,
            ".cir",
        )
    };

    commands.spawn((
        Sprite {
            color: Color::srgb(0.10, 0.12, 0.16),
            custom_size: Some(Vec2::new(480.0, 110.0)),
            ..default()
        },
        Transform::from_xyz(pos_cam.x, pos_cam.y, 20.0),
        DialogoArchivo {
            es_chip,
            nombre: nombre_defecto.clone(),
        },
    ))
    .with_children(|padre| {
        padre.spawn((
            Text2d::new(titulo),
            TextFont {
                font_size: bevy::text::FontSize::Px(14.0),
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.85, 1.0)),
            Transform::from_xyz(0.0, 30.0, 0.1),
        ));

        padre.spawn((
            Sprite {
                color: Color::srgb(0.05, 0.05, 0.07),
                custom_size: Some(Vec2::new(380.0, 38.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -12.0, 0.1),
        ));

        let texto_inicial = format!("{}/{}_{}", carpeta, nombre_defecto, ext);
        padre.spawn((
            Text2d::new(texto_inicial),
            TextFont {
                font_size: bevy::text::FontSize::Px(18.0),
                ..default()
            },
            TextColor(Color::srgb(0.2, 0.95, 0.4)),
            Transform::from_xyz(0.0, -12.0, 0.2),
            TextoInputArchivo,
        ));
    });
}

// --- Bloque 4: Selector Visual de Ficheros .cir en la Carpeta circuitos/ ---
pub fn abrir_selector_carga(
    commands: &mut Commands,
    pos_cam: Vec2,
) {
    asegurar_carpetas();
    let archivos_cir = listar_archivos_por_ext(CARPETA_CIRCUITOS, "cir");

    let ancho_btn = 280.0;
    let alto_btn = 32.0;
    let separacion = 38.0;
    let num_filas = archivos_cir.len().max(1) as f32;
    let total_alto = num_filas * separacion + 70.0;

    commands.spawn((
        Sprite {
            color: Color::srgb(0.10, 0.12, 0.16),
            custom_size: Some(Vec2::new(360.0, total_alto)),
            ..default()
        },
        Transform::from_xyz(pos_cam.x, pos_cam.y, 20.0),
        SelectorCarga,
    ))
    .with_children(|padre| {
        let y_titulo = total_alto / 2.0 - 24.0;

        padre.spawn((
            Text2d::new("CIRCUITOS EN circuitos/ [Esc = Salir]"),
            TextFont {
                font_size: bevy::text::FontSize::Px(14.0),
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.85, 1.0)),
            Transform::from_xyz(0.0, y_titulo, 0.1),
        ));

        if archivos_cir.is_empty() {
            padre.spawn((
                Text2d::new("No hay ficheros .cir en circuitos/"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(15.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.4, 0.4)),
                Transform::from_xyz(0.0, -10.0, 0.1),
            ));
        } else {
            let inicio_y = y_titulo - 38.0;
            for (i, nom_archivo) in archivos_cir.iter().enumerate() {
                let y = inicio_y - (i as f32 * separacion);
                let centro_mundo = Vec2::new(pos_cam.x, pos_cam.y + y);
                let ruta_completa = format!("{}/{}", CARPETA_CIRCUITOS, nom_archivo);

                padre.spawn((
                    Sprite {
                        color: Color::srgb(0.22, 0.26, 0.34),
                        custom_size: Some(Vec2::new(ancho_btn, alto_btn)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, y, 0.1),
                    BotonArchivoCir {
                        nombre_archivo: ruta_completa,
                        centro_mundo,
                    },
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text2d::new(nom_archivo.clone()),
                        TextFont {
                            font_size: bevy::text::FontSize::Px(16.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Transform::from_xyz(0.0, 0.0, 0.1),
                    ));
                });
            }
        }
    });
}

// Lista los archivos dentro de la carpeta indicada ("circuitos" o "chips")
pub fn listar_archivos_por_ext(
    carpeta: &str,
    extension_buscada: &str,
) -> Vec<String> {
    asegurar_carpetas();
    let mut archivos = Vec::new();
    if let Ok(entradas) = fs::read_dir(carpeta) {
        for entrada in entradas.flatten() {
            let ruta = entrada.path();
            if ruta.is_file() {
                if let Some(ext) = ruta.extension() {
                    if ext == extension_buscada {
                        if let Some(nom) = ruta.file_name() {
                            if let Some(nom_str) = nom.to_str() {
                                archivos.push(nom_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    archivos.sort();
    archivos
}

// --- Bloque 5: Interacción con la Lista de Ficheros .cir ---
pub fn gestionar_clic_selector_carga(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    q_selector: Query<Entity, With<SelectorCarga>>,
    mut q_botones_cir: Query<(&BotonArchivoCir, &mut Sprite)>,
    mut commands: Commands,
) {
    let Ok(ent_sel) = q_selector.single() else {
        return;
    };

    if keys.just_pressed(KeyCode::Escape) {
        commands.entity(ent_sel).despawn();
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

    let mut archivo_bajo_cursor = None;
    let mitad_ancho = 140.0;
    let mitad_alto = 16.0;

    for (btn, mut sprite) in q_botones_cir.iter_mut() {
        let pos_btn = btn.centro_mundo;
        let dentro_x = (pos_mundo.x - pos_btn.x).abs() < mitad_ancho;
        let dentro_y = (pos_mundo.y - pos_btn.y).abs() < mitad_alto;

        if dentro_x && dentro_y {
            archivo_bajo_cursor = Some(btn.nombre_archivo.clone());
            sprite.color = Color::srgb(0.35, 0.50, 0.65);
        } else {
            sprite.color = Color::srgb(0.22, 0.26, 0.34);
        }
    }

    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    if let Some(nombre) = archivo_bajo_cursor {
        commands.spawn(PeticionCargar(nombre));
    }

    commands.entity(ent_sel).despawn();
}

// --- Bloque 6: Gestión del Teclado para Guardar en circuitos/ o chips/ ---
pub fn gestionar_teclado_archivo(
    keys: Res<ButtonInput<KeyCode>>,
    mut q_dialogo: Query<(Entity, &mut DialogoArchivo)>,
    mut q_texto: Query<&mut Text2d, With<TextoInputArchivo>>,
    mut commands: Commands,
) {
    let Ok((ent_dlg, mut dlg)) = q_dialogo.single_mut() else {
        return;
    };

    let (carpeta, ext) = if dlg.es_chip {
        (CARPETA_CHIPS, ".chp")
    } else {
        (CARPETA_CIRCUITOS, ".cir")
    };

    if keys.just_pressed(KeyCode::Escape) {
        commands.entity(ent_dlg).despawn();
        return;
    }

    if keys.just_pressed(KeyCode::Enter) {
        let base = dlg.nombre.trim().to_string();
        if !base.is_empty() {
            let nombre_con_ext = if base.ends_with(ext) {
                base
            } else {
                format!("{}{}", base, ext)
            };
            let ruta_final = format!("{}/{}", carpeta, nombre_con_ext);

            commands.spawn(PeticionGuardar(ruta_final));
        }
        commands.entity(ent_dlg).despawn();
        return;
    }

    let mut cambiado = false;
    if keys.just_pressed(KeyCode::Backspace) {
        dlg.nombre.pop();
        cambiado = true;
    }

    let shift = keys.pressed(KeyCode::ShiftLeft) 
        || keys.pressed(KeyCode::ShiftRight);

    let mapa_teclas: [(KeyCode, char, char); 37] = [
        (KeyCode::KeyA, 'a', 'A'),
        (KeyCode::KeyB, 'b', 'B'),
        (KeyCode::KeyC, 'c', 'C'),
        (KeyCode::KeyD, 'd', 'D'),
        (KeyCode::KeyE, 'e', 'E'),
        (KeyCode::KeyF, 'f', 'F'),
        (KeyCode::KeyG, 'g', 'G'),
        (KeyCode::KeyH, 'h', 'H'),
        (KeyCode::KeyI, 'i', 'I'),
        (KeyCode::KeyJ, 'j', 'J'),
        (KeyCode::KeyK, 'k', 'K'),
        (KeyCode::KeyL, 'l', 'L'),
        (KeyCode::KeyM, 'm', 'M'),
        (KeyCode::KeyN, 'n', 'N'),
        (KeyCode::KeyO, 'o', 'O'),
        (KeyCode::KeyP, 'p', 'P'),
        (KeyCode::KeyQ, 'q', 'Q'),
        (KeyCode::KeyR, 'r', 'R'),
        (KeyCode::KeyS, 's', 'S'),
        (KeyCode::KeyT, 't', 'T'),
        (KeyCode::KeyU, 'u', 'U'),
        (KeyCode::KeyV, 'v', 'V'),
        (KeyCode::KeyW, 'w', 'W'),
        (KeyCode::KeyX, 'x', 'X'),
        (KeyCode::KeyY, 'y', 'Y'),
        (KeyCode::KeyZ, 'z', 'Z'),
        (KeyCode::Digit0, '0', '0'),
        (KeyCode::Digit1, '1', '1'),
        (KeyCode::Digit2, '2', '2'),
        (KeyCode::Digit3, '3', '3'),
        (KeyCode::Digit4, '4', '4'),
        (KeyCode::Digit5, '5', '5'),
        (KeyCode::Digit6, '6', '6'),
        (KeyCode::Digit7, '7', '7'),
        (KeyCode::Digit8, '8', '8'),
        (KeyCode::Digit9, '9', '9'),
        (KeyCode::Minus, '-', '_'),
    ];

    for (codigo, min, may) in mapa_teclas.iter() {
        if keys.just_pressed(*codigo) && dlg.nombre.len() < 24 {
            let c = if shift { *may } else { *min };
            dlg.nombre.push(c);
            cambiado = true;
        }
    }

    if cambiado {
        for mut txt in q_texto.iter_mut() {
            txt.0 = format!("{}/{}_{}", carpeta, dlg.nombre, ext);
        }
    }
}

// ==========================================
// Fichero: src/archivo.rs
// Bloque: Bloque 7: Diálogo para Nombrar Pines con la Tecla N (Izquierda en INT / Derecha en LED)
// ==========================================
pub fn gestionar_nombrado_pin(
    keys: Res<ButtonInput<KeyCode>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    q_bloqueos: Query<
        Entity, 
        Or<(With<DialogoArchivo>, With<MenuContextual>, With<SelectorCarga>)>
    >,
    mut q_dlg_pin: Query<(Entity, &mut DialogoEtiquetaPin)>,
    mut q_txt_input: Query<
        &mut Text2d, 
        (With<TextoInputEtiqueta>, Without<TextoEtiquetaNodo>)
    >,
    mut q_nodos_etq: Query<(
        Entity, 
        &Transform, 
        &NodoLogico, 
        &Children, 
        &mut EtiquetaNodo
    )>,
    mut q_txt_nodo: Query<
        (&mut Text2d, &mut Transform), 
        (
            With<TextoEtiquetaNodo>, 
            Without<TextoInputEtiqueta>, 
            Without<FondoEtiquetaNodo>,
            Without<NodoLogico>,
        )
    >,
    mut q_fondo_etq: Query<
        (&mut Sprite, &mut Visibility, &mut Transform), 
        (
            With<FondoEtiquetaNodo>, 
            Without<TextoEtiquetaNodo>,
            Without<NodoLogico>,
        )
    >,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if let Ok((ent_dlg, mut dlg)) = q_dlg_pin.single_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            commands.entity(ent_dlg).despawn();
            return;
        }

        if keys.just_pressed(KeyCode::Enter) {
            let nuevo_nom = dlg.texto.trim().to_uppercase();
            if let Ok((_, _, nodo_log, hijos, mut etq)) = 
                q_nodos_etq.get_mut(dlg.nodo_objetivo) 
            {
                etq.0 = nuevo_nom.clone();
                let nuevo_ancho = ancho_caja_etiqueta(&nuevo_nom);
                let w_agarre = config.ancho_agarre_io;

                // En Interruptores va a la izquierda; en LEDs va a la derecha
                let nuevo_x = if nodo_log.tipo == TipoPuerta::Interruptor {
                    let x_boton = -(w_agarre / 2.0 + 11.0);
                    x_boton - 16.0 - nuevo_ancho / 2.0
                } else {
                    let x_led = w_agarre / 2.0 + 11.0;
                    x_led + 16.0 + nuevo_ancho / 2.0
                };

                for hijo in hijos.iter() {
                    if let Ok((mut txt, mut trf_txt)) = q_txt_nodo.get_mut(hijo) {
                        txt.0 = nuevo_nom.clone();
                        trf_txt.translation.x = nuevo_x;
                    }
                    if let Ok((mut spr_fondo, mut vis_fondo, mut trf_fondo)) = 
                        q_fondo_etq.get_mut(hijo) 
                    {
                        trf_fondo.translation.x = nuevo_x;
                        if nuevo_nom.is_empty() {
                            *vis_fondo = Visibility::Hidden;
                        } else {
                            *vis_fondo = Visibility::Inherited;
                            spr_fondo.custom_size = Some(Vec2::new(nuevo_ancho, 18.0));
                        }
                    }
                }
            }
            commands.entity(ent_dlg).despawn();
            return;
        }

        let mut cambiado = false;
        if keys.just_pressed(KeyCode::Backspace) {
            dlg.texto.pop();
            cambiado = true;
        }

        let teclas: [(KeyCode, char); 37] = [
            (KeyCode::KeyA, 'A'), (KeyCode::KeyB, 'B'), (KeyCode::KeyC, 'C'),
            (KeyCode::KeyD, 'D'), (KeyCode::KeyE, 'E'), (KeyCode::KeyF, 'F'),
            (KeyCode::KeyG, 'G'), (KeyCode::KeyH, 'H'), (KeyCode::KeyI, 'I'),
            (KeyCode::KeyJ, 'J'), (KeyCode::KeyK, 'K'), (KeyCode::KeyL, 'L'),
            (KeyCode::KeyM, 'M'), (KeyCode::KeyN, 'N'), (KeyCode::KeyO, 'O'),
            (KeyCode::KeyP, 'P'), (KeyCode::KeyQ, 'Q'), (KeyCode::KeyR, 'R'),
            (KeyCode::KeyS, 'S'), (KeyCode::KeyT, 'T'), (KeyCode::KeyU, 'U'),
            (KeyCode::KeyV, 'V'), (KeyCode::KeyW, 'W'), (KeyCode::KeyX, 'X'),
            (KeyCode::KeyY, 'Y'), (KeyCode::KeyZ, 'Z'), (KeyCode::Digit0, '0'),
            (KeyCode::Digit1, '1'), (KeyCode::Digit2, '2'), (KeyCode::Digit3, '3'),
            (KeyCode::Digit4, '4'), (KeyCode::Digit5, '5'), (KeyCode::Digit6, '6'),
            (KeyCode::Digit7, '7'), (KeyCode::Digit8, '8'), (KeyCode::Digit9, '9'),
            (KeyCode::Minus, '_'),
        ];

        for (cod, ch) in teclas.iter() {
            if keys.just_pressed(*cod) && dlg.texto.len() < 8 {
                dlg.texto.push(*ch);
                cambiado = true;
            }
        }

        if cambiado {
            for mut txt in q_txt_input.iter_mut() {
                txt.0 = format!("{}_", dlg.texto);
            }
        }
        return;
    }

    if !q_bloqueos.is_empty() || !keys.just_pressed(KeyCode::KeyN) {
        return;
    }

    let Ok(window) = q_windows.single() else { return; };
    let Ok((cam, cam_trf)) = q_camera.single() else { return; };
    let Some(pos_pantalla) = window.cursor_position() else { return; };
    let Ok(pos_mundo) = cam.viewport_to_world_2d(cam_trf, pos_pantalla) else {
        return;
    };
    let pos_cam = cam_trf.translation().truncate();

    for (ent_nodo, trf, _, _, etq) in q_nodos_etq.iter() {
        let centro = trf.translation.truncate();
        if pos_mundo.distance(centro) < config.rad_clic_nodo {
            let actual = etq.0.clone();
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.10, 0.12, 0.16),
                    custom_size: Some(Vec2::new(380.0, 100.0)),
                    ..default()
                },
                Transform::from_xyz(pos_cam.x, pos_cam.y, 20.0),
                DialogoEtiquetaPin {
                    nodo_objetivo: ent_nodo,
                    texto: actual.clone(),
                },
            ))
            .with_children(|padre| {
                padre.spawn((
                    Text2d::new("NOMBRE DEL PIN / CONEXION (Enter = OK, Esc = Salir)"),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.85, 1.0)),
                    Transform::from_xyz(0.0, 26.0, 0.1),
                ));

                padre.spawn((
                    Sprite {
                        color: Color::srgb(0.05, 0.05, 0.07),
                        custom_size: Some(Vec2::new(240.0, 36.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, -12.0, 0.1),
                ));

                padre.spawn((
                    Text2d::new(format!("{}_", actual)),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(18.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.25, 1.0, 0.35)),
                    Transform::from_xyz(0.0, -12.0, 0.2),
                    TextoInputEtiqueta,
                ));
            });
            return;
        }
    }
}

// ==========================================
// Fichero: src/archivo.rs
// Bloque: Bloque 8: Sistema de Guardado, Carga, Inspección y Ctrl+Z
// ==========================================
pub fn procesar_guardado_y_carga(
    keys: Res<ButtonInput<KeyCode>>,
    estado_app: Res<State<ModoApp>>,
    codo_arrastrado: Res<CodoArrastrado>,
    q_arrastrando: Query<Entity, With<Arrastrando>>,
    peticiones: (
        Query<(Entity, &PeticionGuardar)>,
        Query<(Entity, &PeticionCargar)>,
        Query<(Entity, &PeticionEntrarChip)>,
        Query<Entity, With<PeticionSalirChip>>,
    ),
    q_nodos: Query<(
        Entity, 
        &Transform, 
        &NodoLogico, 
        &Children, 
        Option<&PegadoABarra>,
        Option<&InstanciaChip>,
        Option<&EtiquetaNodo>,
    )>,
    q_cables: Query<(Entity, &Cable)>,
    barras_ui: (
        Query<Entity, With<BarraChip>>,
        Query<Entity, With<BarraInferiorChips>>,
        Query<Entity, With<BarraSuperiorInspeccion>>,
        Query<Entity, Or<(With<DialogoArchivo>, With<DialogoEtiquetaPin>)>>,
    ),
    datos_hijos: (
        Query<&BotonInterruptor>,
        Query<&ControlReloj>,
        Query<&TipoConector>,
        Query<&PinEntrada>,
        Query<&PinSalida>,
    ),
    recursos_historial: (
        ResMut<PilaInspeccion>,
        ResMut<HistorialUndo>,
        Res<DesplazamientoBarraChips>,
    ),
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<ConfigApp>,
) {
    let (
        q_guardar, 
        q_cargar, 
        q_entrar, 
        q_salir,
    ) = peticiones;
    let (
        q_barras, 
        q_barra_inf, 
        q_barra_sup, 
        q_dialogo
    ) = barras_ui;
    let (
        q_botones, 
        q_relojes, 
        q_conectores, 
        q_pines_in, 
        q_pines_out
    ) = datos_hijos;
    let (
        mut pila_inspeccion, 
        mut historial_undo,
        scroll_barra,
    ) = recursos_historial;

    let ctrl = keys.pressed(KeyCode::ControlLeft) 
        || keys.pressed(KeyCode::ControlRight);
    if ctrl && keys.just_pressed(KeyCode::KeyZ) && q_dialogo.is_empty() {
        if historial_undo.estados.len() >= 2 {
            historial_undo.estados.pop();
            if let Some(estado_previo) = historial_undo.estados.last().cloned() {
                reconstruir_circuito_desde_texto(
                    &estado_previo,
                    &q_nodos,
                    &q_cables,
                    &q_barras,
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &config,
                );
            }
        }
        return;
    }

    let mut archivo_guardar = None;
    for (ent, pet) in q_guardar.iter() {
        archivo_guardar = Some(pet.0.clone());
        commands.entity(ent).despawn();
    }

    if let Some(ruta) = archivo_guardar {
        guardar_en_disco(
            &ruta,
            &q_nodos,
            &q_cables,
            &q_botones,
            &q_relojes,
            &q_conectores,
            &q_pines_in,
            &q_pines_out,
        );
        if ruta.ends_with(".chp") {
            reconstruir_barra_inferior_chips(
                &mut commands, 
                &q_barra_inf, 
                scroll_barra.indice_inicio
            );
        }
    }

    let mut archivo_cargar = None;
    for (ent, pet) in q_cargar.iter() {
        archivo_cargar = Some(pet.0.clone());
        commands.entity(ent).despawn();
    }

    if let Some(ruta) = archivo_cargar {
        pila_inspeccion.niveles.clear();
        historial_undo.estados.clear();
        reconstruir_barra_superior_inspeccion(
            &mut commands,
            &q_barra_sup,
            &pila_inspeccion,
        );
        cargar_desde_disco(
            &ruta,
            &q_nodos,
            &q_cables,
            &q_barras,
            &mut commands,
            &mut meshes,
            &mut materials,
            &config,
        );
        return;
    }

    let mut chip_a_inspeccionar = None;
    for (ent, pet) in q_entrar.iter() {
        chip_a_inspeccionar = Some(pet.0.clone());
        commands.entity(ent).despawn();
    }

    if let Some(ruta_chp) = chip_a_inspeccionar {
        let nom_base = Path::new(&ruta_chp)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&ruta_chp);

        let ruta_min = format!("{}/{}.chp", CARPETA_CHIPS, nom_base.to_lowercase());
        let ruta_may = format!("{}/{}.chp", CARPETA_CHIPS, nom_base.to_uppercase());

        let contenido_chp = fs::read_to_string(&ruta_chp)
            .or_else(|_| fs::read_to_string(&ruta_min))
            .or_else(|_| fs::read_to_string(&ruta_may));

        if let Ok(texto_chip) = contenido_chp {
            let foto_actual = serializar_circuito_a_texto(
                &q_nodos,
                &q_cables,
                &q_botones,
                &q_relojes,
                &q_conectores,
                &q_pines_in,
                &q_pines_out,
            );
            let nombre_chip = nom_base.to_uppercase();

            pila_inspeccion.niveles.push((nombre_chip, foto_actual));
            historial_undo.estados.clear();
            reconstruir_barra_superior_inspeccion(
                &mut commands,
                &q_barra_sup,
                &pila_inspeccion,
            );

            reconstruir_circuito_desde_texto(
                &texto_chip,
                &q_nodos,
                &q_cables,
                &q_barras,
                &mut commands,
                &mut meshes,
                &mut materials,
                &config,
            );
        } else {
            println!("No se encontró el archivo del chip en chips/: {}", ruta_chp);
        }
        return;
    }

    let mut quiere_salir = false;
    for ent in q_salir.iter() {
        quiere_salir = true;
        commands.entity(ent).despawn();
    }

    if quiere_salir {
        if let Some((_, foto_anterior)) = pila_inspeccion.niveles.pop() {
            historial_undo.estados.clear();
            reconstruir_barra_superior_inspeccion(
                &mut commands,
                &q_barra_sup,
                &pila_inspeccion,
            );
            reconstruir_circuito_desde_texto(
                &foto_anterior,
                &q_nodos,
                &q_cables,
                &q_barras,
                &mut commands,
                &mut meshes,
                &mut materials,
                &config,
            );
        }
        return;
    }

    if q_arrastrando.is_empty() 
        && codo_arrastrado.seleccion.is_none() 
        && *estado_app.get() == ModoApp::Edicion 
    {
        let foto_reposo = serializar_circuito_a_texto(
            &q_nodos,
            &q_cables,
            &q_botones,
            &q_relojes,
            &q_conectores,
            &q_pines_in,
            &q_pines_out,
        );

        if historial_undo.estados.last() != Some(&foto_reposo) {
            if historial_undo.estados.len() >= 50 {
                historial_undo.estados.remove(0);
            }
            historial_undo.estados.push(foto_reposo);
        }
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
        Option<&EtiquetaNodo>,
    )>,
    q_cables: &Query<(Entity, &Cable)>,
    q_botones: &Query<&BotonInterruptor>,
    q_relojes: &Query<&ControlReloj>,
    q_conectores: &Query<&TipoConector>,
    q_pines_in: &Query<&PinEntrada>,
    q_pines_out: &Query<&PinSalida>,
) -> String {
    let mut lineas_nodos = Vec::new();
    let mut lista_nodos = Vec::new();

    let mut nodos_ordenados: Vec<_> = q_nodos.iter().collect();
    nodos_ordenados.sort_by_key(|(ent, _, _, _, _, _, _)| *ent);

    for (ent_nodo, trf, nodo, hijos, opt_pegado, opt_chip, opt_etq) in nodos_ordenados {
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
        Option<&EtiquetaNodo>,
    )>,
    q_cables: &Query<(Entity, &Cable)>,
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
        Option<&EtiquetaNodo>,
    )>,
    q_cables: &Query<(Entity, &Cable)>,
    q_barras: &Query<Entity, With<BarraChip>>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    config: &ConfigApp,
) {
    for (ent_nodo, _, _, _, _, _, _) in q_nodos.iter() {
        commands.entity(ent_nodo).despawn();
    }
    for (ent_cable, _) in q_cables.iter() {
        commands.entity(ent_cable).despawn();
    }
    for ent_b in q_barras.iter() {
        commands.entity(ent_b).despawn();
    }

    let mut necesita_barras = false;
    let mut x_barra_izq: Option<f32> = None;
    let mut x_barra_der: Option<f32> = None;
    let mut max_abs_y: f32 = 400.0;
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

            let (salidas_vec, entradas_vec) = if let Some(nom_chip) = 
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
                    let (_, s_vec, e_vec) = crear_chip_personalizado(
                        commands, 
                        pos, 
                        datos_sub, 
                        config
                    );
                    (s_vec, e_vec)
                } else {
                    (Vec::new(), Vec::new())
                }
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
                    "INT" => {
                        let (ent_i, s_i, e_i) = crear_interruptor(
                            commands, meshes, materials, pos, est, etq_nodo, config
                        );
                        if tag_barra == "PIN_IN" {
                            commands
                                .entity(ent_i)
                                .insert(PegadoABarra(BarraChip::IzquierdaEntradas));
                        }
                        (s_i, e_i)
                    }
                    "LED" => {
                        let (ent_l, s_l, e_l) = crear_led(
                            commands, meshes, materials, pos, etq_nodo, config
                        );
                        if tag_barra == "PIN_OUT" {
                            commands
                                .entity(ent_l)
                                .insert(PegadoABarra(BarraChip::DerechaSalidas));
                        }
                        (s_l, e_l)
                    }
                    "CLK" => crear_reloj(
                        commands, meshes, materials, pos, frec, config
                    ),
                    "D7S" => crear_display(commands, pos, false, config),
                    "D4B" => crear_display(commands, pos, true, config),
                    _ => (None, Vec::new()),
                };
                let v_sal = info_simple.0.into_iter().collect();
                (v_sal, info_simple.1)
            };

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
        Option<&EtiquetaNodo>,
    )>,
    q_cables: &Query<(Entity, &Cable)>,
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
        q_barras,
        commands,
        meshes,
        materials,
        config,
    );

    println!("Circuito cargado correctamente desde {}", ruta_archivo);
}
// ==========================================
// Fichero: src/archivo.rs
// Bloque: Bloque 11: Lector de Estructura Interna de Chip desde chips/
// ==========================================
pub fn leer_datos_chip_desde_chp(ruta_archivo: &str) -> Option<DatosChip> {
    let nom_limpio_path = Path::new(ruta_archivo)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(ruta_archivo);

    let ruta_min = format!(
        "{}/{}.chp", 
        CARPETA_CHIPS, 
        nom_limpio_path.to_lowercase()
    );
    let ruta_may = format!(
        "{}/{}.chp", 
        CARPETA_CHIPS, 
        nom_limpio_path.to_uppercase()
    );

    // Priorizamos siempre la carpeta chips/ para evitar leer versiones antiguas fuera de lugar
    let contenido = fs::read_to_string(&ruta_min)
        .or_else(|_| fs::read_to_string(&ruta_may))
        .or_else(|_| fs::read_to_string(ruta_archivo))
        .ok()?;

    let nombre_limpio = nom_limpio_path.to_uppercase();

    let mut nodos = Vec::new();
    let mut cables = Vec::new();
    let mut entradas_con_y: Vec<(usize, f32, String)> = Vec::new();
    let mut salidas_con_y: Vec<(usize, f32, String)> = Vec::new();

    for linea in contenido.lines() {
        let partes: Vec<&str> = linea.split('|').collect();
        if partes.is_empty() {
            continue;
        }

        if partes[0] == "NODO" && partes.len() >= 6 {
            let cod = partes[1].to_string();
            let x: f32 = partes[2].parse().unwrap_or(0.0);
            let y: f32 = partes[3].parse().unwrap_or(0.0);
            let tag = if partes.len() >= 7 {
                partes[6]
            } else {
                "LIBRE"
            };
            let etq = if partes.len() >= 8 {
                partes[7].to_string()
            } else {
                String::new()
            };

            let idx_actual = nodos.len();

            if tag == "PIN_IN" || (cod == "INT" && (x - (-500.0)).abs() < 20.0) {
                entradas_con_y.push((idx_actual, y, etq));
            } else if tag == "PIN_OUT" || (cod == "LED" && (x - 500.0).abs() < 20.0) {
                salidas_con_y.push((idx_actual, y, etq));
            }

            let sub_chip = if let Some(nom_sub) = cod.strip_prefix("CHIP:") {
                let arch_sub = format!(
                    "{}/{}.chp", 
                    CARPETA_CHIPS, 
                    nom_sub.to_lowercase()
                );
                let arch_sub_may = format!(
                    "{}/{}.chp", 
                    CARPETA_CHIPS, 
                    nom_sub
                );
                leer_datos_chip_desde_chp(&arch_sub)
                    .or_else(|| leer_datos_chip_desde_chp(&arch_sub_may))
                    .map(Box::new)
            } else {
                None
            };

            nodos.push(NodoInternoChip {
                cod_tipo: cod,
                sub_chip,
            });
        } else if partes[0] == "CABLE" && partes.len() >= 4 {
            let orig: usize = partes[1].parse().unwrap_or(0);
            let dest: usize = partes[2].parse().unwrap_or(0);
            let pin_in: usize = partes[3].parse().unwrap_or(0);
            let pin_out: usize = if partes.len() >= 6 {
                partes[5].parse().unwrap_or(0)
            } else {
                0
            };
            cables.push((orig, pin_out, dest, pin_in));
        }
    }

    // Ordenamos de mayor Y (arriba) a menor Y (abajo)
    entradas_con_y.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    salidas_con_y.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut pines_entrada = Vec::new();
    let mut nombres_entrada = Vec::new();
    for (idx, _, nom) in entradas_con_y {
        pines_entrada.push(idx);
        nombres_entrada.push(nom);
    }

    let mut pines_salida = Vec::new();
    let mut nombres_salida = Vec::new();
    for (idx, _, nom) in salidas_con_y {
        pines_salida.push(idx);
        nombres_salida.push(nom);
    }

    if pines_entrada.is_empty() && pines_salida.is_empty() {
        return None;
    }

    Some(DatosChip {
        nombre: nombre_limpio,
        nodos,
        cables,
        pines_entrada,
        nombres_entrada,
        pines_salida,
        nombres_salida,
    })
}