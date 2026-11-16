// ==========================================
// Fichero: src/constructores.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;
use crate::componentes::*;
use crate::configuracion::ConfigApp;

// ==========================================
// Fichero: src/constructores.rs
// Bloque: Bloque 2: Funciones Auxiliares para Conectores y Etiquetas
// ==========================================
pub fn bundle_conector(
    pos: Vec3,
    tipo: TipoConector,
    config: &ConfigApp,
) -> impl Bundle {
    (
        Sprite {
            color: config.col_con_hueco,
            custom_size: Some(config.tam_conector),
            ..default()
        },
        Transform::from_translation(pos),
        tipo,
        ValorSeñal(false),
    )
}

pub fn bundle_hueco(config: &ConfigApp) -> impl Bundle {
    let tam_int = config.tam_conector - Vec2::splat(4.0);
    (
        Sprite {
            color: Color::srgb(0.12, 0.12, 0.15),
            custom_size: Some(tam_int),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.05),
        CentroHueco,
    )
}

// Añade el centro hueco y, si es un conector de SALIDA, dibuja un marco negro interior
// para diferenciar visualmente las salidas de las entradas.
pub fn poblar_interior_conector(
    con: &mut ChildSpawnerCommands,
    tipo: TipoConector,
    config: &ConfigApp,
) {
    con.spawn(bundle_hueco(config));

    if tipo == TipoConector::Salida {
        let tam_marco = config.tam_conector - Vec2::splat(2.0);
        let g = 1.5;
        let offset = (tam_marco.x - g) / 2.0;

        // 4 barritas negras formando un marco interior exclusivo en los conectores de Salida
        con.spawn((
            Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(tam_marco.x, g)),
                ..default()
            },
            Transform::from_xyz(0.0, offset, 0.08),
        ));
        con.spawn((
            Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(tam_marco.x, g)),
                ..default()
            },
            Transform::from_xyz(0.0, -offset, 0.08),
        ));
        con.spawn((
            Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(g, tam_marco.y)),
                ..default()
            },
            Transform::from_xyz(-offset, 0.0, 0.08),
        ));
        con.spawn((
            Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(g, tam_marco.y)),
                ..default()
            },
            Transform::from_xyz(offset, 0.0, 0.08),
        ));
    }
}

// Calcula el ancho del rectángulo negro según la longitud del texto
pub fn ancho_caja_etiqueta(texto: &str) -> f32 {
    (texto.len() as f32 * 9.5 + 14.0).max(28.0)
}

// --- Bloque 3: Geometría Vectorial para Puertas Lógicas ---
fn crear_malla_poligono(
    centro: Vec2, 
    puntos: &[Vec2]
) -> Mesh {
    let mut posiciones = Vec::with_capacity(puntos.len() + 1);
    posiciones.push([centro.x, centro.y, 0.0]);

    for p in puntos {
        posiciones.push([p.x, p.y, 0.0]);
    }

    let mut indices = Vec::with_capacity(puntos.len() * 3);
    let n = puntos.len() as u32;
    for i in 1..=n {
        let sig = if i == n { 1 } else { i + 1 };
        indices.push(0);
        indices.push(i);
        indices.push(sig);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, posiciones);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn puntos_forma_and(
    x_izq: f32,
    x_rect: f32,
    radio: f32,
) -> Vec<Vec2> {
    let mut pts = Vec::new();
    pts.push(Vec2::new(x_izq, radio));
    pts.push(Vec2::new(x_izq, -radio));

    let pasos = 20;
    for i in 0..=pasos {
        let t = i as f32 / pasos as f32;
        let ang = -std::f32::consts::FRAC_PI_2 
            + t * std::f32::consts::PI;
        let x = x_rect + radio * ang.cos();
        let y = radio * ang.sin();
        pts.push(Vec2::new(x, y));
    }
    pts
}

fn puntos_forma_or(
    x_izq: f32,
    x_punta: f32,
    alto_medio: f32,
    curvatura_trasera: f32,
) -> Vec<Vec2> {
    let mut pts = Vec::new();
    let pasos = 16;
    let ancho = x_punta - x_izq;

    for i in 0..=pasos {
        let t = i as f32 / pasos as f32;
        let x = x_izq + t * ancho;
        let y = -alto_medio * (1.0 - t * t);
        pts.push(Vec2::new(x, y));
    }

    for i in 1..=pasos {
        let t = i as f32 / pasos as f32;
        let u = 1.0 - t;
        let x = x_izq + u * ancho;
        let y = alto_medio * (1.0 - u * u);
        pts.push(Vec2::new(x, y));
    }

    for i in 1..pasos {
        let t = i as f32 / pasos as f32;
        let y = alto_medio - t * (2.0 * alto_medio);
        let norm_y = y / alto_medio;
        let x = x_izq + curvatura_trasera * (1.0 - norm_y * norm_y);
        pts.push(Vec2::new(x, y));
    }

    pts
}

fn crear_malla_arco_xor(
    x_base: f32,
    alto_medio: f32,
    curvatura: f32,
    grosor: f32,
) -> Mesh {
    let pasos = 16;
    let mut posiciones = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=pasos {
        let t = i as f32 / pasos as f32;
        let y = alto_medio - t * (2.0 * alto_medio);
        let norm_y = y / alto_medio;
        let x_centro = x_base + curvatura * (1.0 - norm_y * norm_y);

        posiciones.push([x_centro - grosor / 2.0, y, 0.0]);
        posiciones.push([x_centro + grosor / 2.0, y, 0.0]);
    }

    for i in 0..pasos {
        let base = (i * 2) as u32;
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);

        indices.push(base + 1);
        indices.push(base + 3);
        indices.push(base + 2);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, posiciones);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

// ==========================================
// Fichero: src/constructores.rs
// Bloque: Bloque 4: Creador de Puertas Lógicas Vectoriales
// ==========================================
pub fn crear_puerta(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec2,
    tipo_puerta: TipoPuerta,
    config: &ConfigApp,
) -> (Option<Entity>, Vec<Entity>) {
    let texto = match tipo_puerta {
        TipoPuerta::And => "AND",
        TipoPuerta::Or => "OR",
        TipoPuerta::Not => "NOT",
        TipoPuerta::Nand => "NAND",
        TipoPuerta::Nor => "NOR",
        TipoPuerta::Xor => "XOR",
        TipoPuerta::Xnor => "XNOR",
        _ => "GATE",
    };

    let es_negada = matches!(
        tipo_puerta,
        TipoPuerta::Not 
            | TipoPuerta::Nand 
            | TipoPuerta::Nor 
            | TipoPuerta::Xnor
    );
    let es_xor = matches!(
        tipo_puerta, 
        TipoPuerta::Xor | TipoPuerta::Xnor
    );

    let col_borde = config.col_patilla;
    let mat_borde = materials.add(col_borde);
    let mat_relleno = materials.add(Color::WHITE);

    let mut ent_salida = None;
    let mut entradas = Vec::new();

    let x_con_in = -config.tam_puerta.x / 2.0;
    let x_con_out = config.tam_puerta.x / 2.0;

    commands.spawn((
        Transform::from_xyz(pos.x, pos.y, 1.0),
        Visibility::default(),
        NodoLogico { tipo: tipo_puerta },
    ))
    .with_children(|padre| {
        if tipo_puerta == TipoPuerta::Not {
            padre.spawn((
                Sprite {
                    color: col_borde,
                    custom_size: Some(Vec2::new(26.0, 3.5)),
                    ..default()
                },
                Transform::from_xyz(-35.0, 0.0, 0.02),
            ));
        } else {
            padre.spawn((
                Sprite {
                    color: col_borde,
                    custom_size: Some(Vec2::new(28.0, 3.5)),
                    ..default()
                },
                Transform::from_xyz(-34.0, 15.0, 0.02),
            ));
            padre.spawn((
                Sprite {
                    color: col_borde,
                    custom_size: Some(Vec2::new(28.0, 3.5)),
                    ..default()
                },
                Transform::from_xyz(-34.0, -15.0, 0.02),
            ));
        }

        padre.spawn((
            Sprite {
                color: col_borde,
                custom_size: Some(Vec2::new(26.0, 3.5)),
                ..default()
            },
            Transform::from_xyz(35.0, 0.0, 0.02),
        ));

        match tipo_puerta {
            TipoPuerta::Not => {
                let pts_ext = [
                    Vec2::new(-24.0, 24.0),
                    Vec2::new(-24.0, -24.0),
                    Vec2::new(18.0, 0.0),
                ];
                let pts_int = [
                    Vec2::new(-20.5, 19.0),
                    Vec2::new(-20.5, -19.0),
                    Vec2::new(12.0, 0.0),
                ];
                padre.spawn((
                    Mesh2d(meshes.add(crear_malla_poligono(
                        Vec2::new(-10.0, 0.0),
                        &pts_ext,
                    ))),
                    MeshMaterial2d(mat_borde.clone()),
                    Transform::from_xyz(0.0, 0.0, 0.05),
                ));
                padre.spawn((
                    Mesh2d(meshes.add(crear_malla_poligono(
                        Vec2::new(-10.0, 0.0),
                        &pts_int,
                    ))),
                    MeshMaterial2d(mat_relleno.clone()),
                    Transform::from_xyz(0.0, 0.0, 0.10),
                ));
            }
            TipoPuerta::And | TipoPuerta::Nand => {
                let pts_ext = puntos_forma_and(-26.0, 0.0, 25.0);
                let pts_int = puntos_forma_and(-22.5, 0.0, 21.5);

                padre.spawn((
                    Mesh2d(meshes.add(crear_malla_poligono(
                        Vec2::ZERO,
                        &pts_ext,
                    ))),
                    MeshMaterial2d(mat_borde.clone()),
                    Transform::from_xyz(0.0, 0.0, 0.05),
                ));
                padre.spawn((
                    Mesh2d(meshes.add(crear_malla_poligono(
                        Vec2::ZERO,
                        &pts_int,
                    ))),
                    MeshMaterial2d(mat_relleno.clone()),
                    Transform::from_xyz(0.0, 0.0, 0.10),
                ));
            }
            TipoPuerta::Or 
            | TipoPuerta::Nor 
            | TipoPuerta::Xor 
            | TipoPuerta::Xnor => {
                let x_izq = if es_xor { -22.0 } else { -26.0 };
                let pts_ext = puntos_forma_or(x_izq, 25.0, 25.0, 9.0);
                let pts_int = puntos_forma_or(
                    x_izq + 3.5, 
                    20.5, 
                    21.5, 
                    8.0
                );

                padre.spawn((
                    Mesh2d(meshes.add(crear_malla_poligono(
                        Vec2::new(2.0, 0.0),
                        &pts_ext,
                    ))),
                    MeshMaterial2d(mat_borde.clone()),
                    Transform::from_xyz(0.0, 0.0, 0.05),
                ));
                padre.spawn((
                    Mesh2d(meshes.add(crear_malla_poligono(
                        Vec2::new(2.0, 0.0),
                        &pts_int,
                    ))),
                    MeshMaterial2d(mat_relleno.clone()),
                    Transform::from_xyz(0.0, 0.0, 0.10),
                ));

                if es_xor {
                    let malla_arco = crear_malla_arco_xor(
                        -29.0, 
                        25.0, 
                        9.0, 
                        3.5
                    );
                    padre.spawn((
                        Mesh2d(meshes.add(malla_arco)),
                        MeshMaterial2d(mat_borde.clone()),
                        Transform::from_xyz(0.0, 0.0, 0.08),
                    ));
                }
            }
            _ => {}
        }

        if es_negada {
            let x_burbuja = if tipo_puerta == TipoPuerta::Not {
                23.0
            } else {
                30.0
            };
            padre.spawn((
                Mesh2d(meshes.add(Circle::new(5.5))),
                MeshMaterial2d(mat_borde),
                Transform::from_xyz(x_burbuja, 0.0, 0.12),
            ));
            padre.spawn((
                Mesh2d(meshes.add(Circle::new(3.0))),
                MeshMaterial2d(mat_relleno),
                Transform::from_xyz(x_burbuja, 0.0, 0.15),
            ));
        }

        if tipo_puerta != TipoPuerta::Not {
            let x_txt = if es_xor { 1.0 } else { -2.0 };
            padre.spawn((
                Text2d::new(texto),
                TextFont {
                    font_size: bevy::text::FontSize::Px(13.0),
                    ..default()
                },
                TextColor(col_borde),
                Transform::from_xyz(x_txt, 0.0, 0.20),
            ));
        }

        if tipo_puerta == TipoPuerta::Not {
            let e_in = padre.spawn((
                bundle_conector(
                    Vec3::new(x_con_in, 0.0, 0.25),
                    TipoConector::Entrada,
                    config,
                ),
                PinEntrada(0),
            ))
            .with_children(|con| {
                poblar_interior_conector(con, TipoConector::Entrada, config);
            })
            .id();
            entradas.push(e_in);
        } else {
            let e_in1 = padre.spawn((
                bundle_conector(
                    Vec3::new(x_con_in, 15.0, 0.25),
                    TipoConector::Entrada,
                    config,
                ),
                PinEntrada(0),
            ))
            .with_children(|con| {
                poblar_interior_conector(con, TipoConector::Entrada, config);
            })
            .id();
            entradas.push(e_in1);

            let e_in2 = padre.spawn((
                bundle_conector(
                    Vec3::new(x_con_in, -15.0, 0.25),
                    TipoConector::Entrada,
                    config,
                ),
                PinEntrada(1),
            ))
            .with_children(|con| {
                poblar_interior_conector(con, TipoConector::Entrada, config);
            })
            .id();
            entradas.push(e_in2);
        }

        let e_out = padre.spawn(bundle_conector(
            Vec3::new(x_con_out, 0.0, 0.25),
            TipoConector::Salida,
            config,
        ))
        .with_children(|con| {
            poblar_interior_conector(con, TipoConector::Salida, config);
        })
        .id();
        ent_salida = Some(e_out);
    });

    (ent_salida, entradas)
}

// ==========================================
// Fichero: src/constructores.rs
// Bloque: Bloque 5: Creador de Interruptores (Compacto, Agarre Ancho y Nombre a la Izquierda)
// ==========================================
pub fn crear_interruptor(
    commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec2,
    estado_inicial: bool,
    etiqueta: &str,
    config: &ConfigApp,
) -> (Option<Entity>, Vec<Entity>) {
    let color_inicial = if estado_inicial {
        config.col_estado_1
    } else {
        config.col_estado_0
    };

    let mut ent_salida = None;
    let texto_etq = etiqueta.to_string();
    let vis_caja = if texto_etq.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    let ancho_caja = ancho_caja_etiqueta(&texto_etq);

    // Cuerpo central de agarre más ancho (config.ancho_agarre_io) y compacto (config.alto_io)
    let w_agarre = config.ancho_agarre_io;
    let h_io = config.alto_io;
    let x_boton = -(w_agarre / 2.0 + 11.0);
    let x_salida = w_agarre / 2.0 + 6.0;
    // Etiqueta situada a la IZQUIERDA del interruptor
    let x_etiqueta = x_boton - 16.0 - ancho_caja / 2.0;

    commands.spawn((
        Sprite {
            color: config.col_puerta,
            custom_size: Some(Vec2::new(w_agarre, h_io)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 1.0),
        NodoLogico { 
            tipo: TipoPuerta::Interruptor 
        },
        EtiquetaNodo(texto_etq.clone()),
    ))
    .with_children(|padre| {
        // Rectángulo negro a la IZQUIERDA de la entrada
        padre.spawn((
            Sprite {
                color: config.col_etiqueta_fondo,
                custom_size: Some(Vec2::new(ancho_caja, 18.0)),
                ..default()
            },
            Transform::from_xyz(x_etiqueta, 0.0, 0.25),
            vis_caja,
            FondoEtiquetaNodo,
        ));

        // Texto verde claro a la IZQUIERDA de la entrada
        padre.spawn((
            Text2d::new(texto_etq),
            TextFont {
                font_size: bevy::text::FontSize::Px(12.0),
                ..default()
            },
            TextColor(config.col_etiqueta_texto),
            Transform::from_xyz(x_etiqueta, 0.0, 0.30),
            TextoEtiquetaNodo,
        ));

        // Botón interactivo (0 / 1) con altura compacta h_io
        padre.spawn((
            Sprite {
                color: color_inicial,
                custom_size: Some(Vec2::new(20.0, h_io)),
                ..default()
            },
            Transform::from_xyz(x_boton, 0.0, 0.1),
            BotonInterruptor { 
                estado: estado_inicial 
            },
        ));

        // Conector de Salida (con marco negro interior) a la derecha del cuerpo de agarre
        let e_out = padre.spawn(bundle_conector(
            Vec3::new(x_salida, 0.0, 0.1),
            TipoConector::Salida,
            config,
        ))
        .with_children(|con| {
            poblar_interior_conector(con, TipoConector::Salida, config);
        })
        .id();
        ent_salida = Some(e_out);
    });

    (ent_salida, Vec::new())
}

// ==========================================
// Fichero: src/constructores.rs
// Bloque: Bloque 6: Creador de LEDs (Compacto, Agarre Ancho y Nombre a la Derecha)
// ==========================================
pub fn crear_led(
    commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec2,
    etiqueta: &str,
    config: &ConfigApp,
) -> (Option<Entity>, Vec<Entity>) {
    let mut entradas = Vec::new();
    let texto_etq = etiqueta.to_string();
    let vis_caja = if texto_etq.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    let ancho_caja = ancho_caja_etiqueta(&texto_etq);

    // Cuerpo central de agarre más ancho (config.ancho_agarre_io) y compacto (config.alto_io)
    let w_agarre = config.ancho_agarre_io;
    let h_io = config.alto_io;
    let x_entrada = -(w_agarre / 2.0 + 6.0);
    let x_led = w_agarre / 2.0 + 11.0;
    // Etiqueta situada a la DERECHA de la salida (LED)
    let x_etiqueta = x_led + 16.0 + ancho_caja / 2.0;

    commands.spawn((
        Sprite {
            color: config.col_puerta,
            custom_size: Some(Vec2::new(w_agarre, h_io)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 1.0),
        NodoLogico { 
            tipo: TipoPuerta::Led 
        },
        EtiquetaNodo(texto_etq.clone()),
    ))
    .with_children(|padre| {
        // Rectángulo negro a la DERECHA del LED
        padre.spawn((
            Sprite {
                color: config.col_etiqueta_fondo,
                custom_size: Some(Vec2::new(ancho_caja, 18.0)),
                ..default()
            },
            Transform::from_xyz(x_etiqueta, 0.0, 0.25),
            vis_caja,
            FondoEtiquetaNodo,
        ));

        // Texto verde claro a la DERECHA del LED
        padre.spawn((
            Text2d::new(texto_etq),
            TextFont {
                font_size: bevy::text::FontSize::Px(12.0),
                ..default()
            },
            TextColor(config.col_etiqueta_texto),
            Transform::from_xyz(x_etiqueta, 0.0, 0.30),
            TextoEtiquetaNodo,
        ));

        // Indicador luminoso a la derecha con altura compacta h_io
        padre.spawn((
            Sprite {
                color: config.col_estado_0,
                custom_size: Some(Vec2::new(20.0, h_io)),
                ..default()
            },
            Transform::from_xyz(x_led, 0.0, 0.1),
            IndicadorLed,
        ));

        // Conector de Entrada a la izquierda del cuerpo de agarre
        let e_in = padre.spawn((
            bundle_conector(
                Vec3::new(x_entrada, 0.0, 0.1),
                TipoConector::Entrada,
                config,
            ),
            PinEntrada(0),
        ))
        .with_children(|con| {
            poblar_interior_conector(con, TipoConector::Entrada, config);
        })
        .id();
        entradas.push(e_in);
    });

    (None, entradas)
}

// ==========================================
// Fichero: src/constructores.rs
// Bloque: Bloque 7: Creador de Relojes (Clock)
// ==========================================
pub fn crear_reloj(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec2,
    indice_frec_inicial: usize,
    config: &ConfigApp,
) -> (Option<Entity>, Vec<Entity>) {
    let etiquetas = ["OFF", "1Hz", "2Hz", "5Hz", "10Hz"];
    let idx = indice_frec_inicial.min(etiquetas.len() - 1);
    let texto_inicial = etiquetas[idx];

    let mut ent_salida = None;

    commands.spawn((
        Mesh2d(meshes.add(Triangle2d::new(
            Vec2::new(-10.0, 20.0),
            Vec2::new(-10.0, -20.0),
            Vec2::new(22.0, 0.0),
        ))),
        MeshMaterial2d(materials.add(config.col_puerta)),
        Transform::from_xyz(pos.x, pos.y, 1.0),
        NodoLogico { 
            tipo: TipoPuerta::Reloj 
        },
    ))
    .with_children(|padre| {
        padre.spawn((
            Sprite {
                color: config.col_estado_0,
                custom_size: Some(Vec2::new(34.0, 24.0)),
                ..default()
            },
            Transform::from_xyz(-27.0, 0.0, 0.1),
            ControlReloj {
                estado: false,
                indice_frec: idx,
                acumulador: 0.0,
            },
        ))
        .with_children(|cuadro| {
            cuadro.spawn((
                Text2d::new(texto_inicial),
                TextFont {
                    font_size: bevy::text::FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 0.2),
                TextoReloj,
            ));
        });

        let e_out = padre.spawn(bundle_conector(
            Vec3::new(22.0, 0.0, 0.1),
            TipoConector::Salida,
            config,
        ))
        .with_children(|con| {
            poblar_interior_conector(con, TipoConector::Salida, config);
        })
        .id();
        ent_salida = Some(e_out);
    });

    (ent_salida, Vec::new())
}

// --- Bloque 8: Creador de Displays de 7 Segmentos ---
pub fn crear_display(
    commands: &mut Commands,
    pos: Vec2,
    es_4_bits: bool,
    config: &ConfigApp,
) -> (Option<Entity>, Vec<Entity>) {
    let tipo = if es_4_bits {
        TipoPuerta::Display4Bits
    } else {
        TipoPuerta::Display7Seg
    };

    let alto_caja = if es_4_bits { 116.0 } else { 150.0 };
    let ancho_caja = 74.0;
    let mut entradas = Vec::new();

    commands.spawn((
        Sprite {
            color: Color::srgb(0.10, 0.10, 0.12),
            custom_size: Some(Vec2::new(ancho_caja, alto_caja)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 1.0),
        NodoLogico { tipo },
    ))
    .with_children(|padre| {
        if es_4_bits {
            let ys = [36.0, 12.0, -12.0, -36.0];
            for (i, &y) in ys.iter().enumerate() {
                let e_in = padre.spawn((
                    bundle_conector(
                        Vec3::new(-ancho_caja / 2.0, y, 0.1),
                        TipoConector::Entrada,
                        config,
                    ),
                    PinEntrada(i),
                ))
                .with_children(|con| {
                    con.spawn(bundle_hueco(config));
                })
                .id();
                entradas.push(e_in);
            }
        } else {
            let ys = [60.0, 40.0, 20.0, 0.0, -20.0, -40.0, -60.0];
            for (i, &y) in ys.iter().enumerate() {
                let e_in = padre.spawn((
                    bundle_conector(
                        Vec3::new(-ancho_caja / 2.0, y, 0.1),
                        TipoConector::Entrada,
                        config,
                    ),
                    PinEntrada(i),
                ))
                .with_children(|con| {
                    con.spawn(bundle_hueco(config));
                })
                .id();
                entradas.push(e_in);
            }
        }

        let col_apagado = Color::srgb(0.18, 0.18, 0.20);
        let x_off = 5.0;
        let defs_seg: [(usize, Vec2, Vec2); 7] = [
            (0, Vec2::new(x_off, 38.0),        Vec2::new(28.0, 6.0)),
            (1, Vec2::new(x_off + 16.0, 19.0), Vec2::new(6.0, 30.0)),
            (2, Vec2::new(x_off + 16.0, -19.0),Vec2::new(6.0, 30.0)),
            (3, Vec2::new(x_off, -38.0),       Vec2::new(28.0, 6.0)),
            (4, Vec2::new(x_off - 16.0, -19.0),Vec2::new(6.0, 30.0)),
            (5, Vec2::new(x_off - 16.0, 19.0), Vec2::new(6.0, 30.0)),
            (6, Vec2::new(x_off, 0.0),         Vec2::new(28.0, 6.0)),
        ];

        for (idx, pos_seg, tam_seg) in defs_seg.iter() {
            padre.spawn((
                Sprite {
                    color: col_apagado,
                    custom_size: Some(*tam_seg),
                    ..default()
                },
                Transform::from_xyz(pos_seg.x, pos_seg.y, 0.15),
                SegmentoDisplay(*idx),
            ));
        }
    });

    (None, entradas)
}

// ==========================================
// Fichero: src/constructores.rs
// Bloque: Bloque 9: Creador de Barras Laterales para Pines de Chip
// ==========================================
pub fn crear_barras_chip(
    commands: &mut Commands,
    config: &ConfigApp,
) {
    let ancho = 18.0;
    // Altura ampliada a 800px para albergar cómodamente los 18 pines de chips de 8 bits
    let alto = 800.0;
    let x_barra = 500.0;

    commands.spawn((
        Sprite {
            color: config.col_barra_io,
            custom_size: Some(Vec2::new(ancho, alto)),
            ..default()
        },
        Transform::from_xyz(-x_barra, 0.0, 0.5),
        BarraChip::IzquierdaEntradas,
    ))
    .with_children(|padre| {
        padre.spawn((
            Sprite {
                color: config.col_etiqueta_fondo,
                custom_size: Some(Vec2::new(76.0, 22.0)),
                ..default()
            },
            Transform::from_xyz(0.0, alto / 2.0 + 18.0, 0.1),
        ));
        padre.spawn((
            Text2d::new("IN CHIP"),
            TextFont {
                font_size: bevy::text::FontSize::Px(13.0),
                ..default()
            },
            TextColor(config.col_etiqueta_texto),
            Transform::from_xyz(0.0, alto / 2.0 + 18.0, 0.2),
        ));
    });

    commands.spawn((
        Sprite {
            color: config.col_barra_io,
            custom_size: Some(Vec2::new(ancho, alto)),
            ..default()
        },
        Transform::from_xyz(x_barra, 0.0, 0.5),
        BarraChip::DerechaSalidas,
    ))
    .with_children(|padre| {
        padre.spawn((
            Sprite {
                color: config.col_etiqueta_fondo,
                custom_size: Some(Vec2::new(84.0, 22.0)),
                ..default()
            },
            Transform::from_xyz(0.0, alto / 2.0 + 18.0, 0.1),
        ));
        padre.spawn((
            Text2d::new("OUT CHIP"),
            TextFont {
                font_size: bevy::text::FontSize::Px(13.0),
                ..default()
            },
            TextColor(config.col_etiqueta_texto),
            Transform::from_xyz(0.0, alto / 2.0 + 18.0, 0.2),
        ));
    });
}

// ==========================================
// Fichero: src/constructores.rs
// Bloque: Bloque 10: Creador de Chips Personalizados (Pines Alineados al Paso)
// ==========================================
pub fn crear_chip_personalizado(
    commands: &mut Commands,
    pos: Vec2,
    datos: DatosChip,
    config: &ConfigApp,
) -> (Entity, Vec<Entity>, Vec<Entity>) {
    let num_in = datos.pines_entrada.len();
    let num_out = datos.pines_salida.len();
    let max_pines = num_in.max(num_out).max(1);

    let max_len_in = datos
        .nombres_entrada
        .iter()
        .map(|s| s.len())
        .max()
        .unwrap_or(0);
    let max_len_out = datos
        .nombres_salida
        .iter()
        .map(|s| s.len())
        .max()
        .unwrap_or(0);
    let margen_pines = ((max_len_in + max_len_out) as f32 * 8.5).max(28.0);

    let ancho_texto = (datos.nombre.len() as f32 * 11.0).max(76.0);
    let ancho_cuerpo = ancho_texto + margen_pines + 40.0;
    // Usamos config.sep_pines_chip (25px) y alineamos el primer pin a un múltiplo exacto
    // para que todas las patillas coincidan con el snap de los Interruptores y LEDs.
    let separacion_pines = config.sep_pines_chip;
    let alto_cuerpo = (max_pines as f32 * separacion_pines + 30.0).max(64.0);

    let memoria = EstadoRedChip::nuevo_desde_datos(&datos);

    let nombre_mostrar = datos.nombre.clone();
    let nombres_in = datos.nombres_entrada.clone();
    let nombres_out = datos.nombres_salida.clone();
    let archivo_chp = format!("chips/{}.chp", datos.nombre.to_lowercase());
    let mut salidas_ent = Vec::new();
    let mut entradas_ent = Vec::new();

    let ent_chip = commands.spawn((
        Sprite {
            color: Color::srgb(0.18, 0.32, 0.48),
            custom_size: Some(Vec2::new(ancho_cuerpo, alto_cuerpo)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 1.0),
        NodoLogico { tipo: TipoPuerta::Chip },
        InstanciaChip {
            datos,
            memoria,
        },
    ))
    .with_children(|padre| {
        padre.spawn((
            Text2d::new(nombre_mostrar),
            TextFont {
                font_size: bevy::text::FontSize::Px(15.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(0.0, 9.0, 0.2),
        ));

        let y_btn = -alto_cuerpo / 2.0 + 12.0;
        padre.spawn((
            Sprite {
                color: Color::srgb(0.10, 0.18, 0.28),
                custom_size: Some(Vec2::new(42.0, 16.0)),
                ..default()
            },
            Transform::from_xyz(0.0, y_btn, 0.2),
            BotonVerInteriorChip { archivo_chp },
        ))
        .with_children(|btn| {
            btn.spawn((
                Text2d::new("VER"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(10.0),
                    ..default()
                },
                TextColor(Color::srgb(0.65, 0.90, 1.0)),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        });

        // Alineamos el Pin 0 superior a un múltiplo entero de separacion_pines
        let inicio_in_y = 
            ((num_in.saturating_sub(1) / 2) as f32) * separacion_pines;
        for i in 0..num_in {
            let y = inicio_in_y - (i as f32 * separacion_pines);
            let e_in = padre.spawn((
                bundle_conector(
                    Vec3::new(-ancho_cuerpo / 2.0, y, 0.1),
                    TipoConector::Entrada,
                    config,
                ),
                PinEntrada(i),
            ))
            .with_children(|con| {
                poblar_interior_conector(con, TipoConector::Entrada, config);
            })
            .id();
            entradas_ent.push(e_in);

            if let Some(etq) = nombres_in.get(i) {
                if !etq.is_empty() {
                    let w_etq = (etq.len() as f32 * 7.5 + 8.0).max(20.0);
                    let offset_x = 10.0 + w_etq / 2.0;
                    padre.spawn((
                        Sprite {
                            color: config.col_etiqueta_fondo,
                            custom_size: Some(Vec2::new(w_etq, 15.0)),
                            ..default()
                        },
                        Transform::from_xyz(
                            -ancho_cuerpo / 2.0 + offset_x,
                            y,
                            0.22,
                        ),
                    ));
                    padre.spawn((
                        Text2d::new(etq.clone()),
                        TextFont {
                            font_size: bevy::text::FontSize::Px(11.0),
                            ..default()
                        },
                        TextColor(config.col_etiqueta_texto),
                        Transform::from_xyz(
                            -ancho_cuerpo / 2.0 + offset_x,
                            y,
                            0.25,
                        ),
                    ));
                }
            }
        }

        let inicio_out_y = 
            ((num_out.saturating_sub(1) / 2) as f32) * separacion_pines;
        for i in 0..num_out {
            let y = inicio_out_y - (i as f32 * separacion_pines);
            let e_out = padre.spawn((
                bundle_conector(
                    Vec3::new(ancho_cuerpo / 2.0, y, 0.1),
                    TipoConector::Salida,
                    config,
                ),
                PinSalida(i),
            ))
            .with_children(|con| {
                poblar_interior_conector(con, TipoConector::Salida, config);
            })
            .id();
            salidas_ent.push(e_out);

            if let Some(etq) = nombres_out.get(i) {
                if !etq.is_empty() {
                    let w_etq = (etq.len() as f32 * 7.5 + 8.0).max(20.0);
                    let offset_x = 10.0 + w_etq / 2.0;
                    padre.spawn((
                        Sprite {
                            color: config.col_etiqueta_fondo,
                            custom_size: Some(Vec2::new(w_etq, 15.0)),
                            ..default()
                        },
                        Transform::from_xyz(
                            ancho_cuerpo / 2.0 - offset_x,
                            y,
                            0.22,
                        ),
                    ));
                    padre.spawn((
                        Text2d::new(etq.clone()),
                        TextFont {
                            font_size: bevy::text::FontSize::Px(11.0),
                            ..default()
                        },
                        TextColor(config.col_etiqueta_texto),
                        Transform::from_xyz(
                            ancho_cuerpo / 2.0 - offset_x,
                            y,
                            0.25,
                        ),
                    ));
                }
            }
        }
    })
    .id();

    (ent_chip, salidas_ent, entradas_ent)
}