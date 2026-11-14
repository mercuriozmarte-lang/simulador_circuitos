// ==========================================
// Fichero: src/logica.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;
use crate::componentes::*;
use crate::configuracion::ConfigApp;

// --- Bloque 2: Propagación de Cables ---
pub fn propagar_cables(
    q_cables: Query<&Cable>,
    q_conectores: Query<(Entity, &TipoConector)>,
    mut q_valores: Query<&mut ValorSeñal>,
) {
    // 1. Por defecto, toda entrada desconectada empieza en 0 (false)
    for (ent, tipo) in q_conectores.iter() {
        if *tipo == TipoConector::Entrada {
            if let Ok(mut val) = q_valores.get_mut(ent) {
                val.0 = false;
            }
        }
    }

    // 2. Copiamos el valor desde el origen al destino en los cables activos
    for cable in q_cables.iter() {
        if let Ok(val_origen) = q_valores.get(cable.origen) {
            let estado_copiado = val_origen.0;
            
            if let Ok(mut val_destino) = q_valores.get_mut(cable.destino) {
                val_destino.0 = estado_copiado;
            }
        }
    }
}

// --- Bloque 3: Evaluación de Puertas Lógicas y Chips ---
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
        let mut entradas_ordenadas = vec![false; 16];
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

        // Si es un Chip empaquetado, evaluamos su circuito interno
        if puerta.tipo == TipoPuerta::Chip {
            if let Some(mut chip) = opt_chip {
                let InstanciaChip { 
                    ref datos, 
                    ref mut estados_nodos 
                } = *chip;

                let resultados_chip = simular_red_chip(
                    datos,
                    estados_nodos,
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

        // Calculamos el resultado según el tipo de puerta básica
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
            TipoPuerta::Interruptor | TipoPuerta::Reloj => {
                estado_emisor
            }
            TipoPuerta::Led => {
                valores_entradas.first().copied().unwrap_or(false)
            }
            TipoPuerta::Display7Seg 
            | TipoPuerta::Display4Bits 
            | TipoPuerta::Chip => {
                false
            }
        };

        for id_salida in salidas {
            if let Ok((_, mut val_salida)) = q_conectores.get_mut(id_salida) {
                val_salida.0 = resultado;
            }
        }

        for id_led in leds {
            if let Ok(mut sprite_led) = q_leds.get_mut(id_led) {
                if resultado {
                    sprite_led.color = config.col_estado_1;
                } else {
                    sprite_led.color = config.col_estado_0;
                }
            }
        }

        if !segmentos.is_empty() {
            let mascara_seg: [bool; 7] = match puerta.tipo {
                TipoPuerta::Display7Seg => [
                    entradas_ordenadas[0],
                    entradas_ordenadas[1],
                    entradas_ordenadas[2],
                    entradas_ordenadas[3],
                    entradas_ordenadas[4],
                    entradas_ordenadas[5],
                    entradas_ordenadas[6],
                ],
                TipoPuerta::Display4Bits => {
                    let mut valor_hex = 0u8;
                    if entradas_ordenadas[0] { valor_hex += 8; }
                    if entradas_ordenadas[1] { valor_hex += 4; }
                    if entradas_ordenadas[2] { valor_hex += 2; }
                    if entradas_ordenadas[3] { valor_hex += 1; }
                    decodificar_hex_7seg(valor_hex)
                }
                _ => [false; 7],
            };

            let col_apagado = Color::srgb(0.18, 0.18, 0.20);
            for id_seg in segmentos {
                if let Ok((seg_idx, mut sprite_seg)) = 
                    q_segmentos.get_mut(id_seg) 
                {
                    if seg_idx.0 < 7 && mascara_seg[seg_idx.0] {
                        sprite_seg.color = config.col_estado_1;
                    } else {
                        sprite_seg.color = col_apagado;
                    }
                }
            }
        }
    }
}

// Simula el interior de un Chip y devuelve el estado de sus pines de salida
fn simular_red_chip(
    datos: &DatosChip,
    estados_nodos: &mut Vec< Vec<bool> >,
    entradas_ext: &[bool],
) -> Vec<bool> {
    let n_nodos = datos.nodos.len();
    if estados_nodos.len() != n_nodos {
        *estados_nodos = vec![vec![false; 1]; n_nodos];
    }

    // 1. Inyectamos los valores de las patillas de entrada en los Interruptores anclados
    for (idx_pin, &idx_nodo) in datos.pines_entrada.iter().enumerate() {
        let v = entradas_ext.get(idx_pin).copied().unwrap_or(false);
        if idx_nodo < estados_nodos.len() {
            if estados_nodos[idx_nodo].is_empty() {
                estados_nodos[idx_nodo].push(v);
            } else {
                estados_nodos[idx_nodo][0] = v;
            }
        }
    }

    // 2. Iteramos varias pasadas para que la señal atraviese todas las puertas internas
    let pasadas = n_nodos.clamp(2, 12);
    for _ in 0..pasadas {
        for i in 0..n_nodos {
            // Los interruptores de entrada mantienen el valor que viene del exterior
            if datos.pines_entrada.contains(&i) {
                continue;
            }

            // Recogemos los cables que entran al nodo i ordenados por patilla
            let mut in_pines = vec![false; 8];
            let mut max_pin = 0;
            let mut tiene_cables = false;

            for &(orig, pin_out, dest, pin_in) in datos.cables.iter() {
                if dest == i {
                    tiene_cables = true;
                    if pin_in >= in_pines.len() {
                        in_pines.resize(pin_in + 1, false);
                    }
                    if pin_in > max_pin {
                        max_pin = pin_in;
                    }
                    let val_cable = estados_nodos
                        .get(orig)
                        .and_then(|salidas| salidas.get(pin_out))
                        .copied()
                        .unwrap_or(false);
                    in_pines[pin_in] = val_cable;
                }
            }

            let cod = datos.nodos[i].cod_tipo.as_str();

            // Si este nodo interno es a su vez otro sub-chip, lo simulamos recursivamente
            if let Some(sub) = &datos.nodos[i].sub_chip {
                let mut mem_sub = vec![vec![false; 1]; sub.nodos.len()];
                let sal_sub = simular_red_chip(sub, &mut mem_sub, &in_pines);
                estados_nodos[i] = sal_sub;
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
                "LED" => v0,
                _ => false,
            };

            if estados_nodos[i].is_empty() {
                estados_nodos[i].push(res);
            } else {
                estados_nodos[i][0] = res;
            }
        }
    }

    // 3. Recogemos el valor final de los LEDs pegados a la barra derecha
    let mut salidas_finales = Vec::with_capacity(datos.pines_salida.len());
    for &idx_led in datos.pines_salida.iter() {
        let v = estados_nodos
            .get(idx_led)
            .and_then(|s| s.first())
            .copied()
            .unwrap_or(false);
        salidas_finales.push(v);
    }

    salidas_finales
}

// Tabla de verdad para convertir un número de 0 a 15 (0..F) en 7 segmentos (A..G)
fn decodificar_hex_7seg(valor: u8) -> [bool; 7] {
    match valor {
        //   A      B      C      D      E      F      G
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
        10 => [true, true,  true,  false, true,  true,  true ], // A
        11 => [false,false, true,  true,  true,  true,  true ], // b
        12 => [true, false, false, true,  true,  true,  false], // C
        13 => [false,true,  true,  true,  true,  false, true ], // d
        14 => [true, false, false, true,  true,  true,  true ], // E
        15 => [true, false, false, false, true,  true,  true ], // F
        _ => [false; 7],
    }
}

// --- Bloque 4: Feedback Visual (Rojo, Verde o Hueco) ---
pub fn actualizar_visuales_conectores(
    q_cables: Query<&Cable>,
    mut q_conectores: Query<
        (Entity, &ValorSeñal, &mut Sprite, &Children), 
        With<TipoConector>
    >,
    mut q_huecos: Query<&mut Visibility, With<CentroHueco>>,
    config: Res<ConfigApp>,
) {
    for (ent_con, val, mut sprite, hijos) in q_conectores.iter_mut() {
        let conectado = q_cables.iter().any(|c| {
            c.origen == ent_con || c.destino == ent_con
        });

        if conectado {
            if val.0 {
                sprite.color = config.col_estado_1;
            } else {
                sprite.color = config.col_estado_0;
            }
        } else {
            sprite.color = config.col_con_hueco;
        }

        for hijo in hijos.iter() {
            if let Ok(mut vis) = q_huecos.get_mut(hijo) {
                if conectado {
                    *vis = Visibility::Hidden;
                } else {
                    *vis = Visibility::Inherited;
                }
            }
        }
    }
}

// --- Bloque 5: Temporizador de Relojes ---
pub fn actualizar_relojes(
    time: Res<Time>,
    mut q_relojes: Query<(&mut ControlReloj, &mut Sprite)>,
    config: Res<ConfigApp>,
) {
    let frecuencias = [0.0, 1.0, 2.0, 5.0, 10.0];
    let dt = time.delta_secs();

    for (mut reloj, mut sprite) in q_relojes.iter_mut() {
        let frec = frecuencias[reloj.indice_frec];

        if frec <= 0.0 {
            reloj.estado = false;
            reloj.acumulador = 0.0;
            sprite.color = config.col_estado_0;
            continue;
        }

        let semi_periodo = 0.5 / frec;
        reloj.acumulador += dt;

        if reloj.acumulador >= semi_periodo {
            reloj.acumulador -= semi_periodo;
            reloj.estado = !reloj.estado;

            if reloj.estado {
                sprite.color = config.col_estado_1;
            } else {
                sprite.color = config.col_estado_0;
            }
        }
    }
}