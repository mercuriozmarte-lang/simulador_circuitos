// ==========================================
// Fichero: src/archivo.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use std::fs;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::componentes::*;
use crate::configuracion::ConfigApp;
use crate::{
    crear_puerta,
    crear_interruptor,
    crear_led,
    crear_reloj,
    crear_display,
};

// --- Bloque 2: Eventos de Guardado y Carga con Nombre ---
#[derive(Component)]
pub struct PeticionGuardar(pub String);

#[derive(Component)]
pub struct PeticionCargar(pub String);

// --- Bloque 3: Creador del Cuadro de Diálogo (Guardar .cir) ---
pub fn abrir_dialogo_archivo(
    commands: &mut Commands,
    es_guardar: bool,
) {
    let titulo = "GUARDAR CIRCUITO .CIR (Enter = OK, Esc = Cancelar)";
    let nombre_defecto = "circuito".to_string();

    // Contenedor centrado en Z = 20.0 por encima de todo
    commands.spawn((
        Sprite {
            color: Color::srgb(0.10, 0.12, 0.16),
            custom_size: Some(Vec2::new(440.0, 110.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 20.0),
        DialogoArchivo {
            es_guardar,
            nombre: nombre_defecto.clone(),
        },
    ))
    .with_children(|padre| {
        // Título superior de ayuda
        padre.spawn((
            Text2d::new(titulo),
            TextFont {
                font_size: bevy::text::FontSize::Px(14.0),
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.85, 1.0)),
            Transform::from_xyz(0.0, 30.0, 0.1),
        ));

        // Caja oscura de entrada de texto
        padre.spawn((
            Sprite {
                color: Color::srgb(0.05, 0.05, 0.07),
                custom_size: Some(Vec2::new(360.0, 38.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -12.0, 0.1),
        ));

        // Texto que muestra lo que el usuario escribe + extensión .cir
        let texto_inicial = format!("{}_.cir", nombre_defecto);
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

// --- Bloque 4: Selector Visual de Ficheros .cir para Cargar ---
pub fn abrir_selector_carga(commands: &mut Commands) {
    let mut archivos_cir = Vec::new();

    // Leemos el directorio actual y filtramos por la extensión .cir
    if let Ok(entradas) = fs::read_dir(".") {
        for entrada in entradas.flatten() {
            let ruta = entrada.path();
            if ruta.is_file() {
                if let Some(ext) = ruta.extension() {
                    if ext == "cir" {
                        if let Some(nom) = ruta.file_name() {
                            if let Some(nom_str) = nom.to_str() {
                                archivos_cir.push(nom_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    archivos_cir.sort();

    let ancho_btn = 280.0;
    let alto_btn = 32.0;
    let separacion = 38.0;
    let num_filas = archivos_cir.len().max(1) as f32;
    let total_alto = num_filas * separacion + 70.0;

    commands.spawn((
        Sprite {
            color: Color::srgb(0.10, 0.12, 0.16),
            custom_size: Some(Vec2::new(340.0, total_alto)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 20.0),
        SelectorCarga,
    ))
    .with_children(|padre| {
        let y_titulo = total_alto / 2.0 - 24.0;

        padre.spawn((
            Text2d::new("ELIGE UN CIRCUITO (.cir) [Esc = Salir]"),
            TextFont {
                font_size: bevy::text::FontSize::Px(14.0),
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.85, 1.0)),
            Transform::from_xyz(0.0, y_titulo, 0.1),
        ));

        if archivos_cir.is_empty() {
            padre.spawn((
                Text2d::new("No hay ficheros .cir guardados"),
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
                let centro_mundo = Vec2::new(0.0, y);

                padre.spawn((
                    Sprite {
                        color: Color::srgb(0.22, 0.26, 0.34),
                        custom_size: Some(Vec2::new(ancho_btn, alto_btn)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, y, 0.1),
                    BotonArchivoCir {
                        nombre_archivo: nom_archivo.clone(),
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

    // Cerrar con tecla Escape
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

    // Si hace clic izquierdo sobre un fichero .cir, lanzamos la petición de carga
    if let Some(nombre) = archivo_bajo_cursor {
        commands.spawn(PeticionCargar(nombre));
    }

    // Cerramos el selector tras elegir un archivo o al hacer clic fuera
    commands.entity(ent_sel).despawn();
}

// --- Bloque 6: Gestión del Teclado para Guardar Archivo (.cir) ---
pub fn gestionar_teclado_archivo(
    keys: Res<ButtonInput<KeyCode>>,
    mut q_dialogo: Query<(Entity, &mut DialogoArchivo)>,
    mut q_texto: Query<&mut Text2d, With<TextoInputArchivo>>,
    mut commands: Commands,
) {
    let Ok((ent_dlg, mut dlg)) = q_dialogo.single_mut() else {
        return;
    };

    // 1. Cancelar con Escape
    if keys.just_pressed(KeyCode::Escape) {
        commands.entity(ent_dlg).despawn();
        return;
    }

    // 2. Confirmar con Enter
    if keys.just_pressed(KeyCode::Enter) {
        let base = dlg.nombre.trim().to_string();
        if !base.is_empty() {
            let nombre_final = if base.ends_with(".cir") {
                base
            } else {
                format!("{}.cir", base)
            };

            if dlg.es_guardar {
                commands.spawn(PeticionGuardar(nombre_final));
            } else {
                commands.spawn(PeticionCargar(nombre_final));
            }
        }
        commands.entity(ent_dlg).despawn();
        return;
    }

    // 3. Borrar último carácter con Backspace
    let mut cambiado = false;
    if keys.just_pressed(KeyCode::Backspace) {
        dlg.nombre.pop();
        cambiado = true;
    }

    // 4. Lectura de letras, números y guiones
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
            txt.0 = format!("{}_.cir", dlg.nombre);
        }
    }
}
// ==========================================
// Fichero: src/archivo.rs
// Bloque: Bloque 7: Sistema de Guardado y Carga en Fichero
// ==========================================
pub fn procesar_guardado_y_carga(
    q_guardar: Query<(Entity, &PeticionGuardar)>,
    q_cargar: Query<(Entity, &PeticionCargar)>,
    q_nodos: Query<(Entity, &Transform, &NodoLogico, &Children)>,
    q_cables: Query<(Entity, &Cable)>,
    q_botones: Query<&BotonInterruptor>,
    q_relojes: Query<&ControlReloj>,
    q_conectores: Query<&TipoConector>,
    q_pines: Query<&PinEntrada>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<ConfigApp>,
) {
    // 1. Procesar Guardado si existe petición
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
            &q_pines,
        );
    }

    // 2. Procesar Carga si existe petición
    let mut archivo_cargar = None;
    for (ent, pet) in q_cargar.iter() {
        archivo_cargar = Some(pet.0.clone());
        commands.entity(ent).despawn();
    }

    if let Some(ruta) = archivo_cargar {
        cargar_desde_disco(
            &ruta,
            &q_nodos,
            &q_cables,
            &mut commands,
            &mut meshes,
            &mut materials,
            &config,
        );
    }
}

// --- Bloque 8: Lógica de Serialización a Texto ---
fn guardar_en_disco(
    ruta_archivo: &str,
    q_nodos: &Query<(Entity, &Transform, &NodoLogico, &Children)>,
    q_cables: &Query<(Entity, &Cable)>,
    q_botones: &Query<&BotonInterruptor>,
    q_relojes: &Query<&ControlReloj>,
    q_conectores: &Query<&TipoConector>,
    q_pines: &Query<&PinEntrada>,
) {
    let mut lineas = Vec::new();
    let mut lista_nodos = Vec::new();

    // Recorremos todos los nodos y les asignamos un índice numérico (0, 1, 2...)
    for (ent_nodo, trf, nodo, hijos) in q_nodos.iter() {
        let mut estado_bool = 0;
        let mut indice_frec = 1;
        let mut ent_salida = None;
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
                        ent_salida = Some(hijo);
                    }
                    TipoConector::Entrada => {
                        let pin = q_pines
                            .get(hijo)
                            .map(|p| p.0)
                            .unwrap_or(0);
                        entradas_nodo.push((pin, hijo));
                    }
                }
            }
        }

        let cod_tipo = match nodo.tipo {
            TipoPuerta::And => "AND",
            TipoPuerta::Or => "OR",
            TipoPuerta::Not => "NOT",
            TipoPuerta::Nand => "NAND",
            TipoPuerta::Nor => "NOR",
            TipoPuerta::Xor => "XOR",
            TipoPuerta::Xnor => "XNOR",
            TipoPuerta::Interruptor => "INT",
            TipoPuerta::Led => "LED",
            TipoPuerta::Reloj => "CLK",
            TipoPuerta::Display7Seg => "D7S",
            TipoPuerta::Display4Bits => "D4B",
        };

        let x = trf.translation.x;
        let y = trf.translation.y;

        let linea = format!(
            "NODO|{}|{}|{}|{}|{}",
            cod_tipo, x, y, estado_bool, indice_frec
        );
        lineas.push(linea);

        lista_nodos.push((ent_nodo, ent_salida, entradas_nodo));
    }

    // Guardamos los cables indicando de qué índice de nodo a qué índice y pin van
    for (_, cable) in q_cables.iter() {
        let mut idx_origen = None;
        let mut idx_destino = None;
        let mut pin_destino = 0;

        for (i, (_, sal, entradas)) in lista_nodos.iter().enumerate() {
            if *sal == Some(cable.origen) {
                idx_origen = Some(i);
            }
            for (pin, ent_in) in entradas.iter() {
                if *ent_in == cable.destino {
                    idx_destino = Some(i);
                    pin_destino = *pin;
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
                "CABLE|{}|{}|{}|{}",
                orig,
                dest,
                pin_destino,
                codos_str.join(";")
            );
            lineas.push(linea);
        }
    }

    let contenido = lineas.join("\n");
    if let Err(err) = fs::write(ruta_archivo, contenido) {
        println!("Error al guardar {}: {}", ruta_archivo, err);
    } else {
        println!("Circuito guardado correctamente en {}", ruta_archivo);
    }
}

// ==========================================
// Fichero: src/archivo.rs
// Bloque: Bloque 9: Lógica de Lectura y Reconstrucción
// ==========================================
fn cargar_desde_disco(
    ruta_archivo: &str,
    q_nodos: &Query<(Entity, &Transform, &NodoLogico, &Children)>,
    q_cables: &Query<(Entity, &Cable)>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    config: &ConfigApp,
) {
    let Ok(contenido) = fs::read_to_string(ruta_archivo) else {
        println!("No se encontró el fichero {}", ruta_archivo);
        return;
    };

    // 1. Limpiamos el circuito actual de la pantalla
    for (ent_nodo, _, _, _) in q_nodos.iter() {
        commands.entity(ent_nodo).despawn();
    }
    for (ent_cable, _) in q_cables.iter() {
        commands.entity(ent_cable).despawn();
    }

    // Guardaremos ( Option<Entity salida>, Vec<Entity entradas> ) por cada nodo
    let mut conectores_creados: Vec<(Option<Entity>, Vec<Entity>)> = 
        Vec::new();

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
            let pos = Vec2::new(x, y);

            let info_con = match cod {
                "AND" => crear_puerta(
                    commands, 
                    meshes,
                    materials,
                    pos, 
                    TipoPuerta::And, 
                    config
                ),
                "OR" => crear_puerta(
                    commands, 
                    meshes,
                    materials,
                    pos, 
                    TipoPuerta::Or, 
                    config
                ),
                "NOT" => crear_puerta(
                    commands, 
                    meshes,
                    materials,
                    pos, 
                    TipoPuerta::Not, 
                    config
                ),
                "NAND" => crear_puerta(
                    commands, 
                    meshes,
                    materials,
                    pos, 
                    TipoPuerta::Nand, 
                    config
                ),
                "NOR" => crear_puerta(
                    commands, 
                    meshes,
                    materials,
                    pos, 
                    TipoPuerta::Nor, 
                    config
                ),
                "XOR" => crear_puerta(
                    commands, 
                    meshes,
                    materials,
                    pos, 
                    TipoPuerta::Xor, 
                    config
                ),
                "XNOR" => crear_puerta(
                    commands, 
                    meshes,
                    materials,
                    pos, 
                    TipoPuerta::Xnor, 
                    config
                ),
                "INT" => crear_interruptor(
                    commands, 
                    meshes, 
                    materials, 
                    pos, 
                    est, 
                    config
                ),
                "LED" => crear_led(
                    commands, 
                    meshes, 
                    materials, 
                    pos, 
                    config
                ),
                "CLK" => crear_reloj(
                    commands, 
                    meshes, 
                    materials, 
                    pos, 
                    frec, 
                    config
                ),
                "D7S" => crear_display(
                    commands, 
                    pos, 
                    false, 
                    config
                ),
                "D4B" => crear_display(
                    commands, 
                    pos, 
                    true, 
                    config
                ),
                _ => (None, Vec::new()),
            };

            conectores_creados.push(info_con);
        } else if partes[0] == "CABLE" && partes.len() >= 4 {
            let orig_idx: usize = partes[1].parse().unwrap_or(9999);
            let dest_idx: usize = partes[2].parse().unwrap_or(9999);
            let pin_idx: usize = partes[3].parse().unwrap_or(0);

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
                let ent_orig = conectores_creados[orig_idx].0;
                let entradas_dest = &conectores_creados[dest_idx].1;

                if let Some(origen) = ent_orig {
                    if pin_idx < entradas_dest.len() {
                        let destino = entradas_dest[pin_idx];
                        commands.spawn(Cable { 
                            origen, 
                            destino,
                            puntos_intermedios,
                        });
                    }
                }
            }
        }
    }

    println!("Circuito cargado correctamente desde {}", ruta_archivo);
}