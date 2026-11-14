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
    crear_barras_chip,
    crear_chip_personalizado,
};

// --- Bloque 2: Eventos de Guardado, Carga e Inspección de Chips ---
#[derive(Component)]
pub struct PeticionGuardar(pub String);

#[derive(Component)]
pub struct PeticionCargar(pub String);

// NUEVO: Petición para entrar a ver el interior de un archivo .chp
#[derive(Component)]
pub struct PeticionEntrarChip(pub String);

// NUEVO: Petición para volver al circuito anterior de la pila
#[derive(Component)]
pub struct PeticionSalirChip;

// --- Bloque 3: Creador del Cuadro de Diálogo (.cir o .chp) ---
pub fn abrir_dialogo_archivo(
    commands: &mut Commands,
    es_chip: bool,
) {
    let (titulo, nombre_defecto, ext) = if es_chip {
        (
            "GUARDAR CHIP .CHP (Enter = OK, Esc = Cancelar)",
            "mi_chip".to_string(),
            ".chp",
        )
    } else {
        (
            "GUARDAR CIRCUITO .CIR (Enter = OK, Esc = Cancelar)",
            "circuito".to_string(),
            ".cir",
        )
    };

    commands.spawn((
        Sprite {
            color: Color::srgb(0.10, 0.12, 0.16),
            custom_size: Some(Vec2::new(460.0, 110.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 20.0),
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
                custom_size: Some(Vec2::new(360.0, 38.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -12.0, 0.1),
        ));

        let texto_inicial = format!("{}_{}", nombre_defecto, ext);
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
    let archivos_cir = listar_archivos_por_ext("cir");

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

// Lista los archivos del directorio filtrando por extensión ("cir" o "chp")
pub fn listar_archivos_por_ext(extension_buscada: &str) -> Vec<String> {
    let mut archivos = Vec::new();
    if let Ok(entradas) = fs::read_dir(".") {
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

// --- Bloque 6: Gestión del Teclado para Guardar (.cir o .chp) ---
pub fn gestionar_teclado_archivo(
    keys: Res<ButtonInput<KeyCode>>,
    mut q_dialogo: Query<(Entity, &mut DialogoArchivo)>,
    mut q_texto: Query<&mut Text2d, With<TextoInputArchivo>>,
    mut commands: Commands,
) {
    let Ok((ent_dlg, mut dlg)) = q_dialogo.single_mut() else {
        return;
    };

    let ext = if dlg.es_chip { ".chp" } else { ".cir" };

    if keys.just_pressed(KeyCode::Escape) {
        commands.entity(ent_dlg).despawn();
        return;
    }

    if keys.just_pressed(KeyCode::Enter) {
        let base = dlg.nombre.trim().to_string();
        if !base.is_empty() {
            let nombre_final = if base.ends_with(ext) {
                base
            } else {
                format!("{}{}", base, ext)
            };

            commands.spawn(PeticionGuardar(nombre_final));
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
            txt.0 = format!("{}_{}", dlg.nombre, ext);
        }
    }
}

// ==========================================
// Fichero: src/archivo.rs
// Bloque: Bloque 7: Sistema de Guardado, Carga e Inspección de Chips
// ==========================================
pub fn procesar_guardado_y_carga(
    // Agrupamos las 4 peticiones en una tupla para no superar los 16 parámetros de Bevy
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
    )>,
    q_cables: Query<(Entity, &Cable)>,
    q_barras: Query<Entity, With<BarraChip>>,
    q_barra_inf: Query<Entity, With<BarraInferiorChips>>,
    q_barra_sup: Query<Entity, With<BarraSuperiorInspeccion>>,
    q_botones: Query<&BotonInterruptor>,
    q_relojes: Query<&ControlReloj>,
    q_conectores: Query<&TipoConector>,
    q_pines_in: Query<&PinEntrada>,
    q_pines_out: Query<&PinSalida>,
    mut pila_inspeccion: ResMut<PilaInspeccion>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<ConfigApp>,
) {
    let (q_guardar, q_cargar, q_entrar, q_salir) = peticiones;

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
            &q_pines_in,
            &q_pines_out,
        );
        if ruta.ends_with(".chp") {
            reconstruir_barra_inferior_chips(&mut commands, &q_barra_inf);
        }
    }

    // 2. Procesar Carga de fichero .cir si existe petición
    let mut archivo_cargar = None;
    for (ent, pet) in q_cargar.iter() {
        archivo_cargar = Some(pet.0.clone());
        commands.entity(ent).despawn();
    }

    if let Some(ruta) = archivo_cargar {
        pila_inspeccion.niveles.clear();
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
    }

    // 3. Procesar entrada al interior de un Chip ([VER])
    let mut chip_a_inspeccionar = None;
    for (ent, pet) in q_entrar.iter() {
        chip_a_inspeccionar = Some(pet.0.clone());
        commands.entity(ent).despawn();
    }

    if let Some(ruta_chp) = chip_a_inspeccionar {
        // Buscamos el archivo .chp en minúsculas o mayúsculas
        let contenido_chp = fs::read_to_string(&ruta_chp).or_else(|_| {
            let mayus = ruta_chp
                .strip_suffix(".chp")
                .unwrap_or(&ruta_chp)
                .to_uppercase();
            fs::read_to_string(format!("{}.chp", mayus))
        });

        if let Ok(texto_chip) = contenido_chp {
            // Guardamos una fotografía del circuito actual en la pila antes de entrar
            let foto_actual = serializar_circuito_a_texto(
                &q_nodos,
                &q_cables,
                &q_botones,
                &q_relojes,
                &q_conectores,
                &q_pines_in,
                &q_pines_out,
            );
            let nombre_chip = ruta_chp
                .strip_suffix(".chp")
                .unwrap_or(&ruta_chp)
                .to_uppercase();

            pila_inspeccion.niveles.push((nombre_chip, foto_actual));
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
            println!("No se encontró el archivo del chip: {}", ruta_chp);
        }
    }

    // 4. Procesar botón [< VOLVER] para regresar al circuito anterior
    let mut quiere_salir = false;
    for ent in q_salir.iter() {
        quiere_salir = true;
        commands.entity(ent).despawn();
    }

    if quiere_salir {
        if let Some((_, foto_anterior)) = pila_inspeccion.niveles.pop() {
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
    }
}

// --- Bloque 8: Lógica de Serialización a Texto (con Pines de Chip) ---
pub fn serializar_circuito_a_texto(
    q_nodos: &Query<(
        Entity, 
        &Transform, 
        &NodoLogico, 
        &Children, 
        Option<&PegadoABarra>,
        Option<&InstanciaChip>,
    )>,
    q_cables: &Query<(Entity, &Cable)>,
    q_botones: &Query<&BotonInterruptor>,
    q_relojes: &Query<&ControlReloj>,
    q_conectores: &Query<&TipoConector>,
    q_pines_in: &Query<&PinEntrada>,
    q_pines_out: &Query<&PinSalida>,
) -> String {
    let mut lineas = Vec::new();
    let mut lista_nodos = Vec::new();

    for (ent_nodo, trf, nodo, hijos, opt_pegado, opt_chip) in q_nodos.iter() {
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

        let tag_barra = match opt_pegado {
            Some(PegadoABarra(BarraChip::IzquierdaEntradas)) => "PIN_IN",
            Some(PegadoABarra(BarraChip::DerechaSalidas)) => "PIN_OUT",
            None => "LIBRE",
        };

        let x = trf.translation.x;
        let y = trf.translation.y;

        let linea = format!(
            "NODO|{}|{}|{}|{}|{}|{}",
            cod_tipo, x, y, estado_bool, indice_frec, tag_barra
        );
        lineas.push(linea);

        lista_nodos.push((ent_nodo, salidas_nodo, entradas_nodo));
    }

    for (_, cable) in q_cables.iter() {
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
            lineas.push(linea);
        }
    }

    lineas.join("\n")
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
    )>,
    q_cables: &Query<(Entity, &Cable)>,
    q_botones: &Query<&BotonInterruptor>,
    q_relojes: &Query<&ControlReloj>,
    q_conectores: &Query<&TipoConector>,
    q_pines_in: &Query<&PinEntrada>,
    q_pines_out: &Query<&PinSalida>,
) {
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

// --- Bloque 9: Lógica de Lectura y Reconstrucción en Lienzo ---
pub fn reconstruir_circuito_desde_texto(
    contenido: &str,
    q_nodos: &Query<(
        Entity, 
        &Transform, 
        &NodoLogico, 
        &Children, 
        Option<&PegadoABarra>,
        Option<&InstanciaChip>,
    )>,
    q_cables: &Query<(Entity, &Cable)>,
    q_barras: &Query<Entity, With<BarraChip>>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    config: &ConfigApp,
) {
    for (ent_nodo, _, _, _, _, _) in q_nodos.iter() {
        commands.entity(ent_nodo).despawn();
    }
    for (ent_cable, _) in q_cables.iter() {
        commands.entity(ent_cable).despawn();
    }

    let mut necesita_barras = false;
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

            if tag_barra == "PIN_IN" || tag_barra == "PIN_OUT" {
                necesita_barras = true;
            }

            let pos = Vec2::new(x, y);

            let (salidas_vec, entradas_vec) = if let Some(nom_chip) = 
                cod.strip_prefix("CHIP:") 
            {
                let arch_sub_min = format!("{}.chp", nom_chip.to_lowercase());
                let arch_sub_orig = format!("{}.chp", nom_chip);
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
                    "INT" => crear_interruptor(
                        commands, meshes, materials, pos, est, config
                    ),
                    "LED" => crear_led(
                        commands, meshes, materials, pos, config
                    ),
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
        if q_barras.is_empty() {
            crear_barras_chip(commands);
        }
    } else {
        for ent_b in q_barras.iter() {
            commands.entity(ent_b).despawn();
        }
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

// --- Bloque 10: Lector de Estructura Interna de Chip (.chp) ---
pub fn leer_datos_chip_desde_chp(ruta_archivo: &str) -> Option<DatosChip> {
    let contenido = fs::read_to_string(ruta_archivo).ok()?;
    let nombre_limpio = ruta_archivo
        .strip_suffix(".chp")
        .unwrap_or(ruta_archivo)
        .to_uppercase();

    let mut nodos = Vec::new();
    let mut cables = Vec::new();
    let mut entradas_con_y: Vec<(usize, f32)> = Vec::new();
    let mut salidas_con_y: Vec<(usize, f32)> = Vec::new();

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

            let idx_actual = nodos.len();

            if tag == "PIN_IN" || (cod == "INT" && (x - (-500.0)).abs() < 20.0) {
                entradas_con_y.push((idx_actual, y));
            }
            if tag == "PIN_OUT" || (cod == "LED" && (x - 500.0).abs() < 20.0) {
                salidas_con_y.push((idx_actual, y));
            }

            let sub_chip = if let Some(nom_sub) = cod.strip_prefix("CHIP:") {
                let arch_sub = format!("{}.chp", nom_sub.to_lowercase());
                leer_datos_chip_desde_chp(&arch_sub)
                    .or_else(|| leer_datos_chip_desde_chp(&format!("{}.chp", nom_sub)))
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

    entradas_con_y.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    salidas_con_y.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let pines_entrada: Vec<usize> = entradas_con_y.into_iter().map(|(i, _)| i).collect();
    let pines_salida: Vec<usize> = salidas_con_y.into_iter().map(|(i, _)| i).collect();

    if pines_entrada.is_empty() && pines_salida.is_empty() {
        return None;
    }

    Some(DatosChip {
        nombre: nombre_limpio,
        nodos,
        cables,
        pines_entrada,
        pines_salida,
    })
}

// --- Bloque 11: Barra Inferior Flotante de Chips y Barra Superior de Inspección ---
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

// NUEVO: Construye la barra superior con el botón [< VOLVER] cuando estamos dentro de un Chip
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
        // Botón [< VOLVER] situado entre X = 30 y X = 190
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

pub fn gestionar_clic_barra_inferior(
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

    // 1. Comprobar clic en el botón superior [< VOLVER ATRÁS] (top: 10..54, x: 30..190)
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

    // 2. Comprobar clic en los botones de la barra inferior de Chips
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
                    let cuadricula = 50.0;
                    let pos_snap = Vec2::new(
                        (centro_cam.x / cuadricula).round() * cuadricula,
                        (centro_cam.y / cuadricula).round() * cuadricula,
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