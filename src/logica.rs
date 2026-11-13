// ==========================================
// Fichero: src/logica.rs
// ==========================================

use crate::componentes::*;
use crate::configuracion::ConfigApp;
// --- Bloque 1: Importaciones ---
use bevy::prelude::*;

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
        // Obtenemos el valor de la salida origen y se lo pasamos a la entrada destino
        // 1. Leemos y copiamos el booleano (el préstamo inmutable termina en esta línea)
        if let Ok(val_origen) = q_valores.get(cable.origen) {
            let estado_copiado = val_origen.0;

            // 2. Ahora pedimos el préstamo mutable sin conflictos de memoria
            if let Ok(mut val_destino) = q_valores.get_mut(cable.destino) {
                val_destino.0 = estado_copiado;
            }
        }
    }
}

// --- Bloque 3: Evaluación de Puertas Lógicas ---
pub fn evaluar_puertas(
    q_puertas: Query<(Entity, &NodoLogico), With<NodoLogico>>,
    q_children: Query<&Children>,
    q_botones: Query<&BotonInterruptor>,
    // NUEVO: Consultamos el estado de los relojes
    q_relojes: Query<&ControlReloj>,
    mut q_conectores: Query<(&TipoConector, &mut ValorSeñal)>,
    // Consultamos el sprite del LED para iluminarlo
    mut q_leds: Query<&mut Sprite, (With<IndicadorLed>, Without<TipoConector>)>,
    config: Res<ConfigApp>,
) {
    // Recorremos cada puerta lógica del circuito
    for (entidad_puerta, puerta) in q_puertas.iter() {
        let Ok(hijos) = q_children.get(entidad_puerta) else {
            continue;
        };

        // Recopilamos los valores de todas las entradas de esta puerta
        let mut valores_entradas = Vec::new();
        let mut salidas = Vec::new();
        let mut leds = Vec::new();
        let mut estado_emisor = false;

        // En Bevy 0.20, hijos.iter() entrega Entity directamente por valor
        for hijo in hijos.iter() {
            if let Ok((tipo, val)) = q_conectores.get(hijo) {
                match tipo {
                    TipoConector::Entrada => {
                        valores_entradas.push(val.0);
                    }
                    TipoConector::Salida => {
                        salidas.push(hijo);
                    }
                }
            }
            // Si el nodo tiene un botón interruptor hijo, leemos su estado
            if let Ok(boton) = q_botones.get(hijo) {
                estado_emisor = boton.estado;
            }
            // Si el nodo tiene un control de reloj hijo, leemos su pulso actual
            if let Ok(reloj) = q_relojes.get(hijo) {
                estado_emisor = reloj.estado;
            }
            // Si el hijo es la bombilla de un LED, guardamos su entidad
            if q_leds.contains(hijo) {
                leds.push(hijo);
            }
        }

        // Calculamos el resultado según el tipo de puerta (¡soporta N entradas!)
        let resultado = match puerta.tipo {
            TipoPuerta::And => !valores_entradas.is_empty() && valores_entradas.iter().all(|&v| v),
            TipoPuerta::Or => valores_entradas.iter().any(|&v| v),
            TipoPuerta::Not => !valores_entradas.first().copied().unwrap_or(false),
            TipoPuerta::Interruptor | TipoPuerta::Reloj => estado_emisor,
            TipoPuerta::Led => valores_entradas.first().copied().unwrap_or(false),
        };

        // Asignamos el resultado calculado al conector de salida de la puerta
        for id_salida in salidas {
            if let Ok((_, mut val_salida)) = q_conectores.get_mut(id_salida) {
                val_salida.0 = resultado;
            }
        }

        // Actualizamos el color de la bombilla si es un LED
        for id_led in leds {
            if let Ok(mut sprite_led) = q_leds.get_mut(id_led) {
                if resultado {
                    sprite_led.color = config.col_estado_1;
                } else {
                    sprite_led.color = config.col_estado_0;
                }
            }
        }
    }
}

// --- Bloque 4: Feedback Visual (Rojo, Verde o Hueco) ---
pub fn actualizar_visuales_conectores(
    q_cables: Query<&Cable>,
    mut q_conectores: Query<(Entity, &ValorSeñal, &mut Sprite, &Children), With<TipoConector>>,
    mut q_huecos: Query<&mut Visibility, With<CentroHueco>>,
    config: Res<ConfigApp>,
) {
    for (ent_con, val, mut sprite, hijos) in q_conectores.iter_mut() {
        // Comprobamos si este conector tiene algún cable enganchado
        let conectado = q_cables
            .iter()
            .any(|c| c.origen == ent_con || c.destino == ent_con);

        // Si está conectado se rellena de verde (1) o rojo (0).
        // Si NO está conectado, mostramos su centro oscuro para verlo "hueco".
        if conectado {
            if val.0 {
                sprite.color = config.col_estado_1;
            } else {
                sprite.color = config.col_estado_0;
            }
        } else {
            sprite.color = config.col_con_hueco;
        }

        // Usamos hijos.iter() para obtener Entity por valor sin referencias
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

// --- Bloque 5: Temporizador de Relojes (NUEVO) ---
pub fn actualizar_relojes(
    time: Res<Time>,
    mut q_relojes: Query<(&mut ControlReloj, &mut Sprite)>,
    config: Res<ConfigApp>,
) {
    // Tabla de frecuencias disponibles en Hz (0.0 = Apagado)
    let frecuencias = [0.0, 1.0, 2.0, 5.0, 10.0];
    let dt = time.delta_secs();

    for (mut reloj, mut sprite) in q_relojes.iter_mut() {
        let frec = frecuencias[reloj.indice_frec];

        // Si está en 0 Hz, se mantiene apagado en estado 0 (Rojo)
        if frec <= 0.0 {
            reloj.estado = false;
            reloj.acumulador = 0.0;
            sprite.color = config.col_estado_0;
            continue;
        }

        // Medio periodo para alternar entre flanco alto y bajo
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
