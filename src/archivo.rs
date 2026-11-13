// ==========================================
// Fichero: src/archivo.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use std::fs;
use bevy::prelude::*;
use crate::componentes::*;
use crate::configuracion::ConfigApp;
use crate::{
    crear_puerta,
    crear_interruptor,
    crear_led,
    crear_reloj,
    crear_display,
};

// --- Bloque 2: Eventos de Guardado y Carga ---
#[derive(Component)]
pub struct PeticionGuardar;

#[derive(Component)]
pub struct PeticionCargar;

// --- Bloque 3: Sistema de Guardado en Fichero ---
pub fn procesar_guardado_y_carga(
    q_guardar: Query<Entity, With<PeticionGuardar>>,
    q_cargar: Query<Entity, With<PeticionCargar>>,
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
    let mut debe_guardar = false;
    for ent in q_guardar.iter() {
        commands.entity(ent).despawn();
        debe_guardar = true;
    }

    if debe_guardar {
        guardar_en_disco(
            &q_nodos,
            &q_cables,
            &q_botones,
            &q_relojes,
            &q_conectores,
            &q_pines,
        );
    }

    // 2. Procesar Carga si existe petición
    let mut debe_cargar = false;
    for ent in q_cargar.iter() {
        commands.entity(ent).despawn();
        debe_cargar = true;
    }

    if debe_cargar {
        cargar_desde_disco(
            &q_nodos,
            &q_cables,
            &mut commands,
            &mut meshes,
            &mut materials,
            &config,
        );
    }
}

// --- Bloque 4: Lógica de Serialización a Texto ---
fn guardar_en_disco(
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
            let linea = format!(
                "CABLE|{}|{}|{}",
                orig, dest, pin_destino
            );
            lineas.push(linea);
        }
    }

    let contenido = lineas.join("\n");
    if let Err(err) = fs::write("circuito.txt", contenido) {
        println!("Error al guardar circuito.txt: {}", err);
    } else {
        println!("Circuito guardado correctamente en circuito.txt");
    }
}

// --- Bloque 5: Lógica de Lectura y Reconstrucción ---
fn cargar_desde_disco(
    q_nodos: &Query<(Entity, &Transform, &NodoLogico, &Children)>,
    q_cables: &Query<(Entity, &Cable)>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    config: &ConfigApp,
) {
    let Ok(contenido) = fs::read_to_string("circuito.txt") else {
        println!("No se encontró el fichero circuito.txt");
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
                    pos, 
                    TipoPuerta::And, 
                    config
                ),
                "OR" => crear_puerta(
                    commands, 
                    pos, 
                    TipoPuerta::Or, 
                    config
                ),
                "NOT" => crear_puerta(
                    commands, 
                    pos, 
                    TipoPuerta::Not, 
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

            if orig_idx < conectores_creados.len() 
                && dest_idx < conectores_creados.len() 
            {
                let ent_orig = conectores_creados[orig_idx].0;
                let entradas_dest = &conectores_creados[dest_idx].1;

                if let Some(origen) = ent_orig {
                    if pin_idx < entradas_dest.len() {
                        let destino = entradas_dest[pin_idx];
                        commands.spawn(Cable { origen, destino });
                    }
                }
            }
        }
    }

    println!("Circuito cargado correctamente desde circuito.txt");
}