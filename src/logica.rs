// ==========================================
// Fichero: src/logica.rs
// ==========================================

// --- Bloque 1: Importaciones y Temporizador de Relojes ---
use bevy::prelude::*;
use crate::componentes::*;
use crate::configuracion::ConfigApp;

pub fn actualizar_relojes(
    time: Res<Time>,
    mut q_relojes: Query<(&mut ControlReloj, &mut Sprite)>,
    config: Res<ConfigApp>,
) {
    let freqs: [f32; 5] = [0.0, 1.0, 2.0, 5.0, 10.0];
    let dt = time.delta_secs();

    for (mut reloj, mut sprite) in q_relojes.iter_mut() {
        let f = freqs.get(reloj.indice_frec).copied().unwrap_or(0.0);
        if f <= 0.0 {
            reloj.estado = false;
            sprite.color = config.col_estado_0;
            continue;
        }

        let semiperiodo = 0.5 / f;
        reloj.acumulador += dt;

        if reloj.acumulador >= semiperiodo {
            reloj.acumulador -= semiperiodo;
            reloj.estado = !reloj.estado;
            if reloj.estado {
                sprite.color = config.col_estado_1;
            } else {
                sprite.color = config.col_estado_0;
            }
        }
    }
}

// --- Bloque 2: Propagación de Señales y Limpieza de Cables ---
pub fn limpiar_cables_huerfanos(
    q_cables: Query<(Entity, &Cable)>,
    q_conectores: Query<Entity, With<TipoConector>>,
    mut commands: Commands,
) {
    for (ent_cable, cable) in q_cables.iter() {
        if !q_conectores.contains(cable.origen) 
            || !q_conectores.contains(cable.destino) 
        {
            commands.entity(ent_cable).despawn();
        }
    }
}

pub fn propagar_cables(
    q_cables: Query<&Cable>,
    mut q_conectores: Query<(&TipoConector, &mut ValorSeñal)>,
) {
    for (tipo, mut val) in q_conectores.iter_mut() {
        if *tipo == TipoConector::Entrada {
            val.0 = false;
        }
    }

    for cable in q_cables.iter() {
        let valor_origen = q_conectores
            .get(cable.origen)
            .map(|(_, v)| v.0)
            .unwrap_or(false);

        if let Ok((_, mut val_destino)) = 
            q_conectores.get_mut(cable.destino) 
        {
            // Acumulación con |= para permitir buses compartidos gobernados por Tri-State Buffers
            val_destino.0 |= valor_origen;
        }
    }
}

// ==========================================
// Fichero: src/logica.rs
// Bloque: Bloque 3: Evaluación de Puertas Lógicas, Tri-State, Displays y Chips
// ==========================================
pub fn evaluar_puertas(
    mut q_puertas: Query<
        (Entity, &NodoLogico, Option<&mut InstanciaChip>), 
        With<NodoLogico>
    >,
    q_children: Query<&Children>,
    q_botones: Query<&BotonInterruptor>,
    q_relojes: Query<&ControlReloj>,
    q_pines_in: Query<&PinEntrada>,
    q_pines_out: Query<&PinSalida>,
    mut q_conectores: Query<(&TipoConector, &mut ValorSeñal)>,
    mut q_leds: Query<
        &mut Sprite, 
        (
            With<IndicadorLed>, 
            Without<TipoConector>, 
            Without<SegmentoDisplay>
        )
    >,
    mut q_segmentos: Query<
        (&SegmentoDisplay, &mut Sprite),
        (Without<TipoConector>, Without<IndicadorLed>)
    >,
    config: Res<ConfigApp>,
) {
    for (entidad_puerta, puerta, opt_chip) in q_puertas.iter_mut() {
        let Ok(hijos) = q_children.get(entidad_puerta) else { 
            continue; 
        };

        let mut valores_entradas = Vec::new();
        let mut entradas_ordenadas = vec![false; 32];
        let mut salidas = Vec::new();
        let mut leds = Vec::new();
        let mut segmentos = Vec::new();
        let mut estado_emisor = false;

        for hijo in hijos.iter() {
            if let Ok((tipo, val)) = q_conectores.get(hijo) {
                match tipo {
                    TipoConector::Entrada => {
                        valores_entradas.push(val.0);
                        if let Ok(pin) = q_pines_in.get(hijo) {
                            if pin.0 >= entradas_ordenadas.len() {
                                entradas_ordenadas.resize(pin.0 + 1, false);
                            }
                            entradas_ordenadas[pin.0] = val.0;
                        }
                    }
                    TipoConector::Salida => {
                        salidas.push(hijo);
                    }
                }
            }
            if let Ok(boton) = q_botones.get(hijo) {
                estado_emisor = boton.estado;
            }
            if let Ok(reloj) = q_relojes.get(hijo) {
                estado_emisor = reloj.estado;
            }
            if q_leds.contains(hijo) {
                leds.push(hijo);
            }
            if q_segmentos.contains(hijo) {
                segmentos.push(hijo);
            }
        }

        if puerta.tipo == TipoPuerta::Chip {
            if let Some(mut chip) = opt_chip {
                let InstanciaChip { 
                    ref datos, 
                    ref mut memoria 
                } = *chip;

                let resultados_chip = simular_red_chip(
                    datos,
                    memoria,
                    &entradas_ordenadas,
                );

                for id_salida in salidas {
                    let idx_out = q_pines_out
                        .get(id_salida)
                        .map(|p| p.0)
                        .unwrap_or(0);
                    let val_pin = resultados_chip
                        .get(idx_out)
                        .copied()
                        .unwrap_or(false);

                    if let Ok((_, mut val_salida)) = 
                        q_conectores.get_mut(id_salida) 
                    {
                        val_salida.0 = val_pin;
                    }
                }
            }
            continue;
        }

        if puerta.tipo == TipoPuerta::ConectorBus {
            continue;
        }

        let resultado = match puerta.tipo {
            TipoPuerta::And => {
                !valores_entradas.is_empty() 
                    && valores_entradas.iter().all(|&v| v)
            }
            TipoPuerta::Or => {
                valores_entradas.iter().any(|&v| v)
            }
            TipoPuerta::Not => {
                !valores_entradas.first().copied().unwrap_or(false)
            }
            TipoPuerta::Nand => {
                !(
                    !valores_entradas.is_empty() 
                        && valores_entradas.iter().all(|&v| v)
                )
            }
            TipoPuerta::Nor => {
                !valores_entradas.iter().any(|&v| v)
            }
            TipoPuerta::Xor => {
                let unos = valores_entradas
                    .iter()
                    .filter(|&&v| v)
                    .count();
                unos % 2 == 1
            }
            TipoPuerta::Xnor => {
                let unos = valores_entradas
                    .iter()
                    .filter(|&&v| v)
                    .count();
                unos % 2 == 0
            }
            TipoPuerta::TriState => {
                entradas_ordenadas[0] && entradas_ordenadas[1]
            }
            TipoPuerta::Interruptor | TipoPuerta::Reloj => {
                estado_emisor
            }
            TipoPuerta::Led => {
                valores_entradas.first().copied().unwrap_or(false)
            }
            TipoPuerta::Display7Seg 
            | TipoPuerta::Display4Bits 
            | TipoPuerta::ConectorBus
            | TipoPuerta::Chip => {
                false
            }
        };

        for id_salida in salidas {
            if let Ok((_, mut val_salida)) = q_conectores.get_mut(id_salida) {
                val_salida.0 = resultado;
            }
        }

        let estado_indicador = if puerta.tipo == TipoPuerta::TriState {
            entradas_ordenadas[0]
        } else {
            resultado
        };

        for id_led in leds {
            if let Ok(mut sprite_led) = q_leds.get_mut(id_led) {
                if estado_indicador {
                    sprite_led.color = config.col_estado_1;
                } else {
                    sprite_led.color = config.col_estado_0;
                }
            }
        }

        if !segmentos.is_empty() {
            let mascara_seg: [bool; 8] = match puerta.tipo {
                TipoPuerta::Display7Seg => [
                    entradas_ordenadas[1],
                    entradas_ordenadas[2],
                    entradas_ordenadas[3],
                    entradas_ordenadas[4],
                    entradas_ordenadas[5],
                    entradas_ordenadas[6],
                    entradas_ordenadas[7],
                    entradas_ordenadas[0],
                ],
                TipoPuerta::Display4Bits => {
                    let mut valor_hex = 0u8;
                    if entradas_ordenadas[0] { valor_hex += 8; }
                    if entradas_ordenadas[1] { valor_hex += 4; }
                    if entradas_ordenadas[2] { valor_hex += 2; }
                    if entradas_ordenadas[3] { valor_hex += 1; }
                    let seg7 = decodificar_hex_7seg(valor_hex);
                    [
                        seg7[0], seg7[1], seg7[2], seg7[3],
                        seg7[4], seg7[5], seg7[6], false,
                    ]
                }
                _ => [false; 8],
            };

            let col_apagado = Color::srgb(0.18, 0.18, 0.20);
            for id_seg in segmentos {
                if let Ok((seg_idx, mut sprite_seg)) = 
                    q_segmentos.get_mut(id_seg) 
                {
                    if seg_idx.0 < 8 && mascara_seg[seg_idx.0] {
                        sprite_seg.color = config.col_estado_1;
                    } else {
                        sprite_seg.color = col_apagado;
                    }
                }
            }
        }
    }
}

// --- Bloque 4: Simulación Recursiva con Memoria Persistente y Tri-State ("TRI") ---
fn simular_red_chip(
    datos: &DatosChip,
    memoria: &mut EstadoRedChip,
    entradas_ext: &[bool],
) -> Vec<bool> {
    let n_nodos = datos.nodos.len();
    if memoria.salidas_nodos.len() != n_nodos {
        *memoria = EstadoRedChip::nuevo_desde_datos(datos);
    }

    for (idx_pin, &idx_nodo) in datos.pines_entrada.iter().enumerate() {
        let v = entradas_ext.get(idx_pin).copied().unwrap_or(false);
        if idx_nodo < memoria.salidas_nodos.len() {
            if memoria.salidas_nodos[idx_nodo].is_empty() {
                memoria.salidas_nodos[idx_nodo].push(v);
            } else {
                memoria.salidas_nodos[idx_nodo][0] = v;
            }
        }
    }

    let pasadas = 8;
    for _ in 0..pasadas {
        for i in 0..n_nodos {
            if datos.pines_entrada.contains(&i) {
                continue;
            }

            let mut in_pines = vec![false; 32];
            let mut tiene_cables = false;

            for &(orig, pin_out, dest, pin_in) in datos.cables.iter() {
                if dest == i {
                    tiene_cables = true;
                    if pin_in >= in_pines.len() {
                        in_pines.resize(pin_in + 1, false);
                    }
                    let val_cable = memoria
                        .salidas_nodos
                        .get(orig)
                        .and_then(|salidas| salidas.get(pin_out))
                        .copied()
                        .unwrap_or(false);
                    in_pines[pin_in] |= val_cable;
                }
            }

            let cod = datos.nodos[i].cod_tipo.as_str();

            if let Some(sub_datos) = &datos.nodos[i].sub_chip {
                if memoria.sub_redes[i].is_none() {
                    memoria.sub_redes[i] = Some(Box::new(
                        EstadoRedChip::nuevo_desde_datos(sub_datos)
                    ));
                }
                if let Some(sub_mem) = &mut memoria.sub_redes[i] {
                    let sal_sub = simular_red_chip(
                        sub_datos, 
                        sub_mem, 
                        &in_pines
                    );
                    memoria.salidas_nodos[i] = sal_sub;
                }
                continue;
            }

            let v0 = in_pines[0];
            let v1 = in_pines[1];

            let res = match cod {
                "AND" => tiene_cables && v0 && v1,
                "OR" => v0 || v1,
                "NOT" => !v0,
                "NAND" => !(tiene_cables && v0 && v1),
                "NOR" => !(v0 || v1),
                "XOR" => v0 ^ v1,
                "XNOR" => !(v0 ^ v1),
                "TRI" => tiene_cables && v0 && v1,
                "LED" => v0,
                _ => false,
            };

            if memoria.salidas_nodos[i].is_empty() {
                memoria.salidas_nodos[i].push(res);
            } else {
                memoria.salidas_nodos[i][0] = res;
            }
        }
    }

    let mut salidas_finales = Vec::with_capacity(datos.pines_salida.len());
    for &idx_led in datos.pines_salida.iter() {
        let v = memoria
            .salidas_nodos
            .get(idx_led)
            .and_then(|s| s.first())
            .copied()
            .unwrap_or(false);
        salidas_finales.push(v);
    }

    salidas_finales
}

fn decodificar_hex_7seg(valor: u8) -> [bool; 7] {
    match valor {
        0 => [true,  true,  true,  true,  true,  true,  false],
        1 => [false, true,  true,  false, false, false, false],
        2 => [true,  true,  false, true,  true,  false, true ],
        3 => [true,  true,  true,  true,  false, false, true ],
        4 => [false, true,  true,  false, false, true,  true ],
        5 => [true,  false, true,  true,  false, true,  true ],
        6 => [true,  false, true,  true,  true,  true,  true ],
        7 => [true,  true,  true,  false, false, false, false],
        8 => [true,  true,  true,  true,  true,  true,  true ],
        9 => [true,  true,  true,  true,  false, true,  true ],
        10 => [true, true,  true,  false, true,  true,  true ],
        11 => [false,false, true,  true,  true,  true,  true ],
        12 => [true, false, false, true,  true,  true,  false],
        13 => [false,true,  true,  true,  true,  false, true ],
        14 => [true, false, false, true,  true,  true,  true ],
        15 => [true, false, false, false, true,  true,  true ],
        _ => [false; 7],
    }
}

// --- Bloque 5: Actualización Visual de Conectores (Huecos vs Conectados) ---
pub fn actualizar_visuales_conectores(
    q_cables: Query<&Cable>,
    mut q_conectores: Query<(
        Entity, 
        &ValorSeñal, 
        &mut Sprite, 
        &Children
    )>,
    mut q_huecos: Query<&mut Visibility, With<CentroHueco>>,
    config: Res<ConfigApp>,
) {
    for (ent_con, val_senal, mut sprite_con, hijos) in q_conectores.iter_mut() {
        let esta_conectado = q_cables.iter().any(|c| {
            c.origen == ent_con || c.destino == ent_con
        });

        if esta_conectado {
            if val_senal.0 {
                sprite_con.color = config.col_estado_1;
            } else {
                sprite_con.color = config.col_estado_0;
            }
        } else {
            sprite_con.color = config.col_con_hueco;
        }

        for hijo in hijos.iter() {
            if let Ok(mut vis_hueco) = q_huecos.get_mut(hijo) {
                if esta_conectado {
                    *vis_hueco = Visibility::Hidden;
                } else {
                    *vis_hueco = Visibility::Inherited;
                }
            }
        }
    }
}