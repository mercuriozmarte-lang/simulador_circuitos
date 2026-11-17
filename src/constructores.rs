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
// Bloque: Bloque 4: Creador de Puertas Lógicas y Buffer Tri-State (con Punto Enable)
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
        TipoPuerta::TriState => "TRI",
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
        if tipo_puerta == TipoPuerta::TriState {
            // 1. Pista horizontal de la patilla superior (Pin 0 = Enable) hasta X = -6.0
            padre.spawn((
                Sprite {
                    color: col_borde,
                    custom_size: Some(Vec2::new(42.0, 3.5)),
                    ..default()
                },
                Transform::from_xyz(-27.0, 18.0, 0.02),
            ));
            // Bajada vertical hacia el punto de control en el lomo superior del triángulo
            padre.spawn((
                Sprite {
                    color: col_borde,
                    custom_size: Some(Vec2::new(3.5, 9.0)),
                    ..default()
                },
                Transform::from_xyz(-6.0, 14.0, 0.02),
            ));

            // PUNTO / SEÑAL DE ENABLE sobre el borde oblicuo superior del triángulo:
            // Aro exterior oscuro + Punto interior que cambia de rojo (0) a verde (1) según EN
            padre.spawn((
                Sprite {
                    color: col_borde,
                    custom_size: Some(Vec2::new(10.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(-6.0, 10.0, 0.14),
            ));
            padre.spawn((
                Sprite {
                    color: config.col_estado_0,
                    custom_size: Some(Vec2::new(6.0, 6.0)),
                    ..default()
                },
                Transform::from_xyz(-6.0, 10.0, 0.18),
                IndicadorLed,
            ));

            // Etiqueta "EN" junto al pin superior de habilitación
            padre.spawn((
                Text2d::new("EN"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(10.0),
                    ..default()
                },
                TextColor(col_borde),
                Transform::from_xyz(-34.0, 25.0, 0.20),
            ));

            // 2. Patilla inferior (Pin 1 = Data): entra a la base izquierda del triángulo
            padre.spawn((
                Sprite {
                    color: col_borde,
                    custom_size: Some(Vec2::new(26.0, 3.5)),
                    ..default()
                },
                Transform::from_xyz(-35.0, -12.5, 0.02),
            ));

            // 3. Patilla de salida derecha
            padre.spawn((
                Sprite {
                    color: col_borde,
                    custom_size: Some(Vec2::new(26.0, 3.5)),
                    ..default()
                },
                Transform::from_xyz(35.0, 0.0, 0.02),
            ));

            // 4. Cuerpo triangular vectorial
            let pts_ext = [
                Vec2::new(-24.0, 22.0),
                Vec2::new(-24.0, -24.0),
                Vec2::new(24.0, 0.0),
            ];
            let pts_int = [
                Vec2::new(-20.5, 16.5),
                Vec2::new(-20.5, -18.5),
                Vec2::new(17.0, 0.0),
            ];

            padre.spawn((
                Mesh2d(meshes.add(crear_malla_poligono(
                    Vec2::new(-6.0, 0.0),
                    &pts_ext,
                ))),
                MeshMaterial2d(mat_borde.clone()),
                Transform::from_xyz(0.0, 0.0, 0.05),
            ));
            padre.spawn((
                Mesh2d(meshes.add(crear_malla_poligono(
                    Vec2::new(-6.0, 0.0),
                    &pts_int,
                ))),
                MeshMaterial2d(mat_relleno.clone()),
                Transform::from_xyz(0.0, 0.0, 0.10),
            ));

            padre.spawn((
                Text2d::new("TRI"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(11.0),
                    ..default()
                },
                TextColor(col_borde),
                Transform::from_xyz(-10.0, -3.0, 0.20),
            ));

            let e_en = padre.spawn((
                bundle_conector(
                    Vec3::new(x_con_in, 18.0, 0.25),
                    TipoConector::Entrada,
                    config,
                ),
                PinEntrada(0),
            ))
            .with_children(|con| {
                poblar_interior_conector(con, TipoConector::Entrada, config);
            })
            .id();
            entradas.push(e_en);

            let e_dt = padre.spawn((
                bundle_conector(
                    Vec3::new(x_con_in, -12.5, 0.25),
                    TipoConector::Entrada,
                    config,
                ),
                PinEntrada(1),
            ))
            .with_children(|con| {
                poblar_interior_conector(con, TipoConector::Entrada, config);
            })
            .id();
            entradas.push(e_dt);

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
            return;
        }

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
) -> (Entity, Option<Entity>, Vec<Entity>) {
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

    let w_agarre = config.ancho_agarre_io;
    let h_io = config.alto_io;
    let x_boton = -(w_agarre / 2.0 + 11.0);
    let x_salida = w_agarre / 2.0 + 6.0;
    let x_etiqueta = x_boton - 16.0 - ancho_caja / 2.0;

    let ent_int = commands.spawn((
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
    })
    .id();

    (ent_int, ent_salida, Vec::new())
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
) -> (Entity, Option<Entity>, Vec<Entity>) {
    let mut entradas = Vec::new();
    let texto_etq = etiqueta.to_string();
    let vis_caja = if texto_etq.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    let ancho_caja = ancho_caja_etiqueta(&texto_etq);

    let w_agarre = config.ancho_agarre_io;
    let h_io = config.alto_io;
    let x_entrada = -(w_agarre / 2.0 + 6.0);
    let x_led = w_agarre / 2.0 + 11.0;
    let x_etiqueta = x_led + 16.0 + ancho_caja / 2.0;

    let ent_led = commands.spawn((
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

        padre.spawn((
            Sprite {
                color: config.col_estado_0,
                custom_size: Some(Vec2::new(20.0, h_io)),
                ..default()
            },
            Transform::from_xyz(x_led, 0.0, 0.1),
            IndicadorLed,
        ));

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
    })
    .id();

    (ent_led, None, entradas)
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

// ==========================================
// Fichero: src/constructores.rs
// Bloque: Bloque 8: Creador de Displays Apilables en Vertical (Margen = 0.5 * Paso)
// ==========================================
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

    let sep = config.sep_pines_chip; // 25.0px
    let num_pines = if es_4_bits { 4 } else { 8 };
    // Altura total = N * 25.0px (100.0px para 4 bits, 200.0px para 7 segmentos)
    let alto_caja = (num_pines as f32) * sep;
    let ancho_caja = if es_4_bits { 74.0 } else { 104.0 };
    let mut entradas = Vec::new();

    // Primer pin a (alto_caja / 2.0) - (0.5 * sep):
    // - En 4 bits (alto 100, borde sup +50.0): pines en +37.5, +12.5, -12.5, -37.5 (margen sup e inf = 12.5px)
    // - En 7 seg (alto 200, borde sup +100.0): pines en +87.5 .. -87.5 (margen sup e inf = 12.5px)
    let inicio_y = (alto_caja / 2.0) - (sep * 0.5);

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
            for i in 0..4 {
                let y = inicio_y - (i as f32 * sep);
                let e_in = padre.spawn((
                    bundle_conector(
                        Vec3::new(-ancho_caja / 2.0, y, 0.1),
                        TipoConector::Entrada,
                        config,
                    ),
                    PinEntrada(i),
                ))
                .with_children(|con| {
                    poblar_interior_conector(con, TipoConector::Entrada, config);
                })
                .id();
                entradas.push(e_in);
            }
        } else {
            let nombres_pines = ["S", "A", "B", "C", "D", "E", "F", "G"];
            for (i, nom_pin) in nombres_pines.iter().enumerate() {
                let y = inicio_y - (i as f32 * sep);
                let e_in = padre.spawn((
                    bundle_conector(
                        Vec3::new(-ancho_caja / 2.0, y, 0.1),
                        TipoConector::Entrada,
                        config,
                    ),
                    PinEntrada(i),
                ))
                .with_children(|con| {
                    poblar_interior_conector(con, TipoConector::Entrada, config);
                })
                .id();
                entradas.push(e_in);

                padre.spawn((
                    Text2d::new(*nom_pin),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(config.col_etiqueta_texto),
                    Transform::from_xyz(-ancho_caja / 2.0 + 16.0, y, 0.2),
                ));
            }
        }

        let col_apagado = Color::srgb(0.18, 0.18, 0.20);
        let x_off = if es_4_bits { 5.0 } else { 18.0 };

        let defs_seg: [(usize, Vec2, Vec2); 7] = [
            (0, Vec2::new(x_off, 34.0),        Vec2::new(28.0, 6.0)),
            (1, Vec2::new(x_off + 16.0, 17.0), Vec2::new(6.0, 26.0)),
            (2, Vec2::new(x_off + 16.0, -17.0),Vec2::new(6.0, 26.0)),
            (3, Vec2::new(x_off, -34.0),       Vec2::new(28.0, 6.0)),
            (4, Vec2::new(x_off - 16.0, -17.0),Vec2::new(6.0, 26.0)),
            (5, Vec2::new(x_off - 16.0, 17.0), Vec2::new(6.0, 26.0)),
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

        if !es_4_bits {
            padre.spawn((
                Sprite {
                    color: col_apagado,
                    custom_size: Some(Vec2::new(14.0, 6.0)),
                    ..default()
                },
                Transform::from_xyz(x_off - 32.0, 0.0, 0.15),
                SegmentoDisplay(7),
            ));
        }
    });

    (None, entradas)
}

// ==========================================
// Fichero: src/constructores.rs
// Bloque: Bloque 9: Creador de Barras Laterales con Asas de Movimiento y Altura
// ==========================================
pub fn crear_barras_chip(
    commands: &mut Commands,
    config: &ConfigApp,
) {
    crear_barras_chip_con_dimensiones(
        commands, 
        config, 
        -500.0, 
        500.0, 
        800.0
    );
}

pub fn crear_barras_chip_con_dimensiones(
    commands: &mut Commands,
    config: &ConfigApp,
    x_izq: f32,
    x_der: f32,
    alto: f32,
) {
    let ancho = 18.0;
    let alto_final = alto.max(200.0);

    let defs = [
        (BarraChip::IzquierdaEntradas, x_izq, "IN CHIP"),
        (BarraChip::DerechaSalidas, x_der, "OUT CHIP"),
    ];

    for (tipo_barra, pos_x, titulo) in defs {
        commands.spawn((
            Sprite {
                color: config.col_barra_io,
                custom_size: Some(Vec2::new(ancho, alto_final)),
                ..default()
            },
            Transform::from_xyz(pos_x, 0.0, 0.5),
            tipo_barra,
        ))
        .with_children(|padre| {
            // 1. Etiqueta superior identificativa (IN CHIP / OUT CHIP)
            padre.spawn((
                Sprite {
                    color: config.col_etiqueta_fondo,
                    custom_size: Some(Vec2::new(84.0, 20.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, alto_final / 2.0 + 38.0, 0.1),
                EtiquetaSuperiorBarra,
            ))
            .with_children(|cartel| {
                cartel.spawn((
                    Text2d::new(titulo),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(12.0),
                        ..default()
                    },
                    TextColor(config.col_etiqueta_texto),
                    Transform::from_xyz(0.0, 0.0, 0.1),
                ));
            });

            // 2. ASA SUPERIOR: Estirar / Encoger altura de la barra (^ ALTO v)
            padre.spawn((
                Sprite {
                    color: Color::srgb(0.18, 0.36, 0.58),
                    custom_size: Some(Vec2::new(82.0, 22.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, alto_final / 2.0 + 12.0, 0.2),
                AsaAltoBarra(tipo_barra),
            ))
            .with_children(|asa| {
                asa.spawn((
                    Text2d::new("^ ALTO v"),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 0.0, 0.1),
                ));
            });

            // 3. ASA INFERIOR: Mover la barra horizontalmente (< MOVER >)
            padre.spawn((
                Sprite {
                    color: Color::srgb(0.55, 0.36, 0.14),
                    custom_size: Some(Vec2::new(86.0, 22.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, -alto_final / 2.0 - 12.0, 0.2),
                AsaMoverBarra(tipo_barra),
            ))
            .with_children(|asa| {
                asa.spawn((
                    Text2d::new("< MOVER >"),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(11.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 0.0, 0.1),
                ));
            });
        });
    }
}

// ==========================================
// Fichero: src/constructores.rs
// Bloque: Bloque 10: Creador de Chips Personalizados y Conector de Bus
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

    let sep = config.sep_pines_chip;
    let alto_caja = (max_pines as f32 * sep + 36.0).max(90.0);

    let max_len_in = datos
        .nombres_entrada
        .iter()
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(0);
    let max_len_out = datos
        .nombres_salida
        .iter()
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(0);

    let w_in = if max_len_in > 0 {
        (max_len_in as f32) * 7.5 + 16.0
    } else {
        0.0
    };
    let w_out = if max_len_out > 0 {
        (max_len_out as f32) * 7.5 + 16.0
    } else {
        0.0
    };
    let w_titulo = (datos.nombre.chars().count() as f32) * 9.5 + 30.0;

    let ancho_caja = (w_in + w_titulo + w_out + 36.0).max(120.0);

    let mut salidas = Vec::new();
    let mut entradas = Vec::new();

    let nombre_titulo = datos.nombre.clone();
    let archivo_origen = format!(
        "chips/{}.chp", 
        datos.nombre.to_lowercase()
    );
    let memoria_inicial = EstadoRedChip::nuevo_desde_datos(&datos);

    let entidad_chip = commands.spawn((
        Sprite {
            color: Color::srgb(0.18, 0.30, 0.45),
            custom_size: Some(Vec2::new(ancho_caja, alto_caja)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 1.0),
        NodoLogico { 
            tipo: TipoPuerta::Chip 
        },
        InstanciaChip {
            datos: datos.clone(),
            memoria: memoria_inicial,
        },
    ))
    .with_children(|padre| {
        padre.spawn((
            Text2d::new(nombre_titulo),
            TextFont {
                font_size: bevy::text::FontSize::Px(14.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(0.0, 6.0, 0.2),
        ));

        padre.spawn((
            Sprite {
                color: Color::srgb(0.10, 0.18, 0.28),
                custom_size: Some(Vec2::new(40.0, 16.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -alto_caja / 2.0 + 12.0, 0.2),
            BotonVerInteriorChip {
                archivo_chp: archivo_origen,
            },
        ))
        .with_children(|btn| {
            btn.spawn((
                Text2d::new("VER"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(10.0),
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.90, 1.0)),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        });

        let inicio_in = ((num_in / 2) as f32) * sep;
        for i in 0..num_in {
            let y = inicio_in - (i as f32 * sep);
            let e_in = padre.spawn((
                bundle_conector(
                    Vec3::new(-ancho_caja / 2.0, y, 0.1),
                    TipoConector::Entrada,
                    config,
                ),
                PinEntrada(i),
            ))
            .with_children(|con| {
                poblar_interior_conector(
                    con, 
                    TipoConector::Entrada, 
                    config
                );
            })
            .id();
            entradas.push(e_in);

            if let Some(nom_in) = datos.nombres_entrada.get(i) {
                if !nom_in.is_empty() {
                    let w_box = ancho_caja_etiqueta(nom_in);
                    let x_box = -ancho_caja / 2.0 + 10.0 + w_box / 2.0;
                    padre.spawn((
                        Sprite {
                            color: config.col_etiqueta_fondo,
                            custom_size: Some(Vec2::new(w_box, 16.0)),
                            ..default()
                        },
                        Transform::from_xyz(x_box, y, 0.15),
                    ));
                    padre.spawn((
                        Text2d::new(nom_in.clone()),
                        TextFont {
                            font_size: bevy::text::FontSize::Px(11.0),
                            ..default()
                        },
                        TextColor(config.col_etiqueta_texto),
                        Transform::from_xyz(x_box, y, 0.20),
                    ));
                }
            }
        }

        let inicio_out = ((num_out / 2) as f32) * sep;
        for j in 0..num_out {
            let y = inicio_out - (j as f32 * sep);
            let e_out = padre.spawn((
                bundle_conector(
                    Vec3::new(ancho_caja / 2.0, y, 0.1),
                    TipoConector::Salida,
                    config,
                ),
                PinSalida(j),
            ))
            .with_children(|con| {
                poblar_interior_conector(
                    con, 
                    TipoConector::Salida, 
                    config
                );
            })
            .id();
            salidas.push(e_out);

            if let Some(nom_out) = datos.nombres_salida.get(j) {
                if !nom_out.is_empty() {
                    let w_box = ancho_caja_etiqueta(nom_out);
                    let x_box = ancho_caja / 2.0 - 10.0 - w_box / 2.0;
                    padre.spawn((
                        Sprite {
                            color: config.col_etiqueta_fondo,
                            custom_size: Some(Vec2::new(w_box, 16.0)),
                            ..default()
                        },
                        Transform::from_xyz(x_box, y, 0.15),
                    ));
                    padre.spawn((
                        Text2d::new(nom_out.clone()),
                        TextFont {
                            font_size: bevy::text::FontSize::Px(11.0),
                            ..default()
                        },
                        TextColor(config.col_etiqueta_texto),
                        Transform::from_xyz(x_box, y, 0.20),
                    ));
                }
            }
        }
    })
    .id();

    (entidad_chip, salidas, entradas)
}

// ==========================================
// Fichero: src/constructores.rs
// Bloque: Bloque 10 (Función crear_conector_bus)
// ==========================================
// Constructor del Conector de Bus (N patillas a 25px, zona de agarre, punto rojo en Pin 0, giro 0..3, flecha y regulador +/-)
pub fn crear_conector_bus(
    commands: &mut Commands,
    pos: Vec2,
    datos_bus: DatosConectorBus,
    config: &ConfigApp,
) -> (Entity, Vec<Entity>, Vec<Entity>) {
    let n = datos_bus.num_hilos.clamp(2, 16);
    let sep = config.sep_pines_chip; // 25.0px
    let alto_barra = (n as f32) * sep;
    let ancho_barra = 22.0;
    let inicio_y = (alto_barra / 2.0) - (sep * 0.5);

    let rot = datos_bus.rotacion % 4;
    // Si está girado 180° (2) o 270° (3), invertimos el eje Y local para que el Pin 0 (punto rojo)
    // siga quedando arriba/izquierda en pantalla y los cables no se crucen.
    let signo_y = if rot == 2 || rot == 3 { -1.0 } else { 1.0 };

    let angulo = (rot as f32) * std::f32::consts::FRAC_PI_2;
    let mut trf_padre = Transform::from_xyz(pos.x, pos.y, 1.0);
    trf_padre.rotation = Quat::from_rotation_z(angulo);

    let mut salidas = Vec::new();
    let mut entradas = Vec::new();

    let tipo_patilla = match datos_bus.modo {
        ModoDireccionBus::HaciaBus => TipoConector::Entrada,
        ModoDireccionBus::DesdeBus => TipoConector::Salida,
        ModoDireccionBus::Bidireccional => TipoConector::Salida,
    };
    let es_bidireccional = 
        datos_bus.modo == ModoDireccionBus::Bidireccional;

    let grosor_cuello = ((n as f32) * 2.0 + 4.0).clamp(12.0, 28.0);
    let largo_cuello = 36.0;
    let x_centro_cuello = ancho_barra / 2.0 + largo_cuello / 2.0;
    let x_fin_cuello = ancho_barra / 2.0 + largo_cuello;

    let ent_padre = commands.spawn((
        Sprite {
            color: Color::srgb(0.12, 0.14, 0.18),
            custom_size: Some(Vec2::new(ancho_barra, alto_barra)),
            ..default()
        },
        trf_padre,
        NodoLogico {
            tipo: TipoPuerta::ConectorBus,
        },
        DatosConectorBus {
            num_hilos: n,
            rotacion: rot,
            modo: datos_bus.modo,
        },
    ))
    .id();

    commands.entity(ent_padre).with_children(|padre| {
        // 1. Interior blanco del cuerpo rectangular
        padre.spawn((
            Sprite {
                color: Color::srgb(0.95, 0.96, 0.98),
                custom_size: Some(Vec2::new(
                    ancho_barra - 4.0, 
                    alto_barra - 4.0
                )),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.05),
        ));

        // 1B. ZONA DE AGARRE DIFERENCIADA
        let alto_agarre = (alto_barra - 22.0).max(24.0);
        padre.spawn((
            Sprite {
                color: Color::srgb(0.72, 0.78, 0.86),
                custom_size: Some(Vec2::new(10.0, alto_agarre)),
                ..default()
            },
            Transform::from_xyz(3.0, 0.0, 0.09),
        ))
        .with_children(|agarre| {
            for dy in [-6.0f32, 0.0, 6.0] {
                agarre.spawn((
                    Sprite {
                        color: Color::srgb(0.32, 0.38, 0.46),
                        custom_size: Some(Vec2::new(6.0, 2.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, dy, 0.02),
                ));
            }
        });

        // 2. Punto ROJO indicador del Hilo 1 (Pin 0) siempre en el extremo del Pin 0
        padre.spawn((
            Sprite {
                color: Color::srgb(0.92, 0.15, 0.15),
                custom_size: Some(Vec2::new(7.0, 7.0)),
                ..default()
            },
            Transform::from_xyz(3.0, inicio_y * signo_y, 0.15),
        ));

        // 3. Las N patillas en el borde con paso de 25px
        for i in 0..n {
            let y = (inicio_y - (i as f32 * sep)) * signo_y;
            let pos_pin = Vec3::new(-ancho_barra / 2.0 - 4.0, y, 0.20);

            if tipo_patilla == TipoConector::Entrada {
                let e_in = padre.spawn((
                    bundle_conector(
                        pos_pin, 
                        TipoConector::Entrada, 
                        config
                    ),
                    PinEntrada(i),
                ))
                .with_children(|con| {
                    poblar_interior_conector(
                        con, 
                        TipoConector::Entrada, 
                        config
                    );
                })
                .id();
                entradas.push(e_in);
            } else if es_bidireccional {
                let e_bi = padre.spawn((
                    bundle_conector(
                        pos_pin, 
                        TipoConector::Salida, 
                        config
                    ),
                    PinEntrada(i),
                    PinSalida(i),
                    ConectorBidireccional,
                ))
                .with_children(|con| {
                    poblar_interior_conector(
                        con, 
                        TipoConector::Salida, 
                        config
                    );
                })
                .id();
                salidas.push(e_bi);
                entradas.push(e_bi);
            } else {
                let e_out = padre.spawn((
                    bundle_conector(
                        pos_pin, 
                        TipoConector::Salida, 
                        config
                    ),
                    PinSalida(i),
                ))
                .with_children(|con| {
                    poblar_interior_conector(
                        con, 
                        TipoConector::Salida, 
                        config
                    );
                })
                .id();
                salidas.push(e_out);
            }
        }

        // 4. Cuello negro de salida hacia el Bus (con flecha de dirección)
        let mut trf_texto_flecha = Transform::from_xyz(-3.0, 0.0, 0.20);
        if rot == 2 {
            trf_texto_flecha.rotation = 
                Quat::from_rotation_z(std::f32::consts::PI);
        }
        let txt_flecha = if rot == 2 {
            match datos_bus.modo {
                ModoDireccionBus::HaciaBus => "<-",
                ModoDireccionBus::DesdeBus => "->",
                ModoDireccionBus::Bidireccional => "<->",
            }
        } else {
            datos_bus.modo.texto_flecha()
        };

        padre.spawn((
            Sprite {
                color: Color::srgb(0.10, 0.12, 0.15),
                custom_size: Some(Vec2::new(
                    largo_cuello, 
                    grosor_cuello
                )),
                ..default()
            },
            Transform::from_xyz(x_centro_cuello, 0.0, 0.08),
            ControlFlechaBus {
                nodo_padre: ent_padre,
            },
        ))
        .with_children(|cuello| {
            cuello.spawn((
                Text2d::new(txt_flecha),
                TextFont {
                    font_size: bevy::text::FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.85, 0.25)),
                trf_texto_flecha,
            ));
        });

        // 5. Regulador interactivo [+] y [-]
        let y_reg = (grosor_cuello / 2.0 + 11.0) * signo_y;
        padre.spawn((
            Sprite {
                color: Color::srgb(0.20, 0.45, 0.30),
                custom_size: Some(Vec2::new(14.0, 14.0)),
                ..default()
            },
            Transform::from_xyz(x_centro_cuello + 5.0, y_reg, 0.25),
            BotonReguladorBus {
                nodo_padre: ent_padre,
                delta: 1,
            },
        ))
        .with_children(|b| {
            b.spawn((
                Text2d::new("+"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        });

        padre.spawn((
            Text2d::new(format!("{}", n)),
            TextFont {
                font_size: bevy::text::FontSize::Px(11.0),
                ..default()
            },
            TextColor(Color::srgb(0.15, 0.20, 0.30)),
            Transform::from_xyz(x_centro_cuello - 9.0, y_reg, 0.25),
        ));

        padre.spawn((
            Sprite {
                color: Color::srgb(0.50, 0.22, 0.22),
                custom_size: Some(Vec2::new(14.0, 14.0)),
                ..default()
            },
            Transform::from_xyz(x_centro_cuello + 5.0, -y_reg, 0.25),
            BotonReguladorBus {
                nodo_padre: ent_padre,
                delta: -1,
            },
        ))
        .with_children(|b| {
            b.spawn((
                Text2d::new("-"),
                TextFont {
                    font_size: bevy::text::FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        });

        // 6. Anillo circular de conexión al tronco del Bus
        padre.spawn((
            Sprite {
                color: Color::srgb(0.10, 0.12, 0.15),
                custom_size: Some(Vec2::new(16.0, 16.0)),
                ..default()
            },
            Transform::from_xyz(x_fin_cuello, 0.0, 0.22),
            PuertoTroncoBus {
                nodo_padre: ent_padre,
            },
        ))
        .with_children(|puerto| {
            puerto.spawn((
                Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(10.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.05),
            ));
            puerto.spawn((
                Sprite {
                    color: Color::srgb(0.10, 0.12, 0.15),
                    custom_size: Some(Vec2::new(5.0, 5.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.10),
            ));
        });
    });

    (ent_padre, salidas, entradas)
}