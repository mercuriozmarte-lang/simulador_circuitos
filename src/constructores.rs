// ==========================================
// Fichero: src/constructores.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;
use crate::componentes::*;
use crate::configuracion::ConfigApp;

// --- Bloque 2: Constructores de Conectores ---
pub fn bundle_conector(
    pos: Vec3,
    tipo: TipoConector,
    config: &ConfigApp,
) -> (Sprite, Transform, TipoConector, ValorSeñal) {
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

pub fn bundle_hueco(
    config: &ConfigApp
) -> (Sprite, Transform, CentroHueco) {
    // Cuadrado interior oscuro que hace que el conector se vea "hueco"
    (
        Sprite {
            color: Color::srgb(0.1, 0.1, 0.1),
            custom_size: Some(
                config.tam_conector - Vec2::splat(4.0)
            ),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.1),
        CentroHueco,
    )
}


// Bloque: Bloque 3: Creador de Puertas Lógicas Vectoriales
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;

// Convierte un polígono convexo o en abanico en una malla 2D de triángulos
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

// Genera los puntos del contorno de una puerta AND / NAND (forma en D)
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

// Genera los puntos del contorno curvo de una puerta OR / NOR / XOR / XNOR
fn puntos_forma_or(
    x_izq: f32,
    x_punta: f32,
    alto_medio: f32,
    curvatura_trasera: f32,
) -> Vec<Vec2> {
    let mut pts = Vec::new();
    let pasos = 16;
    let ancho = x_punta - x_izq;

    // 1. Curva inferior (desde la esquina inferior izquierda hasta la punta derecha)
    for i in 0..=pasos {
        let t = i as f32 / pasos as f32;
        let x = x_izq + t * ancho;
        let y = -alto_medio * (1.0 - t * t);
        pts.push(Vec2::new(x, y));
    }

    // 2. Curva superior (desde la punta derecha hasta la esquina superior izquierda)
    for i in 1..=pasos {
        let t = i as f32 / pasos as f32;
        let u = 1.0 - t;
        let x = x_izq + u * ancho;
        let y = alto_medio * (1.0 - u * u);
        pts.push(Vec2::new(x, y));
    }

    // 3. Arco cóncavo trasero (de arriba a abajo por el lado izquierdo)
    for i in 1..pasos {
        let t = i as f32 / pasos as f32;
        let y = alto_medio - t * (2.0 * alto_medio);
        let norm_y = y / alto_medio;
        let x = x_izq + curvatura_trasera * (1.0 - norm_y * norm_y);
        pts.push(Vec2::new(x, y));
    }

    pts
}

// Genera una cinta curva para el arco extra de las puertas XOR y XNOR
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

pub fn crear_puerta(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec2,
    tipo_puerta: TipoPuerta,
    config: &ConfigApp,
) -> (Option<Entity>, Vec<Entity>) {
    // Texto identificador de la puerta
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

    let mat_borde = materials.add(Color::BLACK);
    let mat_relleno = materials.add(Color::WHITE);

    let mut ent_salida = None;
    let mut entradas = Vec::new();

    // Posición X de los conectores en los extremos (-50 y +50)
    let x_con_in = -config.tam_puerta.x / 2.0;
    let x_con_out = config.tam_puerta.x / 2.0;

    commands.spawn((
        Transform::from_xyz(pos.x, pos.y, 1.0),
        Visibility::default(),
        NodoLogico { tipo: tipo_puerta },
    ))
    .with_children(|padre| {
        // 1. Dibujamos las líneas negras de las patillas (pines metálicos)
        if tipo_puerta == TipoPuerta::Not {
            padre.spawn((
                Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(26.0, 3.5)),
                    ..default()
                },
                Transform::from_xyz(-35.0, 0.0, 0.02),
            ));
        } else {
            // Patilla superior (Y = +15.0)
            padre.spawn((
                Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(28.0, 3.5)),
                    ..default()
                },
                Transform::from_xyz(-34.0, 15.0, 0.02),
            ));
            // Patilla inferior (Y = -15.0)
            padre.spawn((
                Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(28.0, 3.5)),
                    ..default()
                },
                Transform::from_xyz(-34.0, -15.0, 0.02),
            ));
        }

        // Patilla de salida derecha (Y = 0.0)
        padre.spawn((
            Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(26.0, 3.5)),
                ..default()
            },
            Transform::from_xyz(35.0, 0.0, 0.02),
        ));

        // 2. Cuerpo vectorial según la familia de puerta (Borde negro + Relleno blanco)
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

                // Si es XOR o XNOR, añadimos el arco curvo independiente detrás
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

        // 3. Si la puerta es negada (NOT, NAND, NOR, XNOR), añadimos el circulito inversor
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

        // 4. Etiqueta de texto en negro nítido en el centro de la puerta
        if tipo_puerta != TipoPuerta::Not {
            let x_txt = if es_xor { 1.0 } else { -2.0 };
            padre.spawn((
                Text2d::new(texto),
                TextFont {
                    font_size: bevy::text::FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::BLACK),
                Transform::from_xyz(x_txt, 0.0, 0.20),
            ));
        }

        // 5. Conectores interactivos de Entrada y Salida
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
                con.spawn(bundle_hueco(config));
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
                con.spawn(bundle_hueco(config));
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
                con.spawn(bundle_hueco(config));
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
            con.spawn(bundle_hueco(config));
        })
        .id();
        ent_salida = Some(e_out);
    });

    (ent_salida, entradas)
}

// --- Bloque 4: Creador de Interruptores ---
pub fn crear_interruptor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec2,
    estado_inicial: bool,
    config: &ConfigApp,
) -> (Option<Entity>, Vec<Entity>) {
    // Malla triangular apuntando a la derecha
    let malla_triangulo = meshes.add(Triangle2d::new(
        Vec2::new(0.0, 15.0),
        Vec2::new(0.0, -15.0),
        Vec2::new(18.0, 0.0),
    ));
    let mat_cuerpo = materials.add(config.col_puerta);
    let col_btn = if estado_inicial {
        config.col_estado_1
    } else {
        config.col_estado_0
    };

    let mut ent_salida = None;

    commands.spawn((
        Transform::from_xyz(pos.x, pos.y, 1.0),
        Visibility::default(),
        NodoLogico { tipo: TipoPuerta::Interruptor },
    ))
    .with_children(|padre| {
        // 1. Primer cuadrado (Izquierda): Botón para alternar 0 (Rojo) y 1 (Verde)
        padre.spawn((
            Sprite {
                color: col_btn,
                custom_size: Some(Vec2::new(26.0, 26.0)),
                ..default()
            },
            Transform::from_xyz(-28.0, 0.0, 0.1),
            BotonInterruptor { estado: estado_inicial },
        ));

        // 2. Segundo cuadrado (Centro, junto al primero): Cuerpo para moverlo
        padre.spawn((
            Sprite {
                color: config.col_puerta,
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // 3. Triángulo (Derecha, pegado al segundo cuadrado): Parte del cuerpo
        padre.spawn((
            Mesh2d(malla_triangulo),
            MeshMaterial2d(mat_cuerpo),
            Transform::from_xyz(15.0, 0.0, 0.0),
        ));

        // 4. Conector de Salida en la punta del triángulo
        let e_out = padre.spawn(bundle_conector(
            Vec3::new(33.0, 0.0, 0.2),
            TipoConector::Salida,
            config,
        ))
        .with_children(|con| {
            con.spawn(bundle_hueco(config));
        })
        .id();
        ent_salida = Some(e_out);
    });

    (ent_salida, Vec::new())
}

// --- Bloque 5: Creador de LEDs de Salida ---
pub fn crear_led(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec2,
    config: &ConfigApp,
) -> (Option<Entity>, Vec<Entity>) {
    // Malla triangular apuntando a la izquierda (hacia el conector de entrada)
    let malla_triangulo = meshes.add(Triangle2d::new(
        Vec2::new(0.0, 15.0),
        Vec2::new(0.0, -15.0),
        Vec2::new(-18.0, 0.0),
    ));
    let mat_cuerpo = materials.add(config.col_puerta);
    let mut entradas = Vec::new();

    commands.spawn((
        Transform::from_xyz(pos.x, pos.y, 1.0),
        Visibility::default(),
        NodoLogico { tipo: TipoPuerta::Led },
    ))
    .with_children(|padre| {
        // 1. Conector de Entrada en la punta izquierda del triángulo
        let e_in = padre.spawn((
            bundle_conector(
                Vec3::new(-33.0, 0.0, 0.2),
                TipoConector::Entrada,
                config,
            ),
            PinEntrada(0),
        ))
        .with_children(|con| {
            con.spawn(bundle_hueco(config));
        })
        .id();
        entradas.push(e_in);

        // 2. Triángulo (Izquierda, pegado al cuerpo central)
        padre.spawn((
            Mesh2d(malla_triangulo),
            MeshMaterial2d(mat_cuerpo),
            Transform::from_xyz(-15.0, 0.0, 0.0),
        ));

        // 3. Cuadrado central: Cuerpo para arrastrar el LED
        padre.spawn((
            Sprite {
                color: config.col_puerta,
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // 4. Cuadrado derecho: Bombilla que se ilumina en Rojo (0) o Verde (1)
        padre.spawn((
            Sprite {
                color: config.col_estado_0,
                custom_size: Some(Vec2::new(26.0, 26.0)),
                ..default()
            },
            Transform::from_xyz(28.0, 0.0, 0.1),
            IndicadorLed,
        ));
    });

    (None, entradas)
}

// --- Bloque 6: Creador de Relojes (Clock) ---
pub fn crear_reloj(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec2,
    indice_frec_inicial: usize,
    config: &ConfigApp,
) -> (Option<Entity>, Vec<Entity>) {
    // Malla triangular apuntando a la derecha
    let malla_triangulo = meshes.add(Triangle2d::new(
        Vec2::new(0.0, 15.0),
        Vec2::new(0.0, -15.0),
        Vec2::new(18.0, 0.0),
    ));
    let mat_cuerpo = materials.add(config.col_puerta);
    let etiquetas_frec = ["OFF", "1Hz", "2Hz", "5Hz", "10Hz"];
    let idx = indice_frec_inicial % etiquetas_frec.len();

    let mut ent_salida = None;

    commands.spawn((
        Transform::from_xyz(pos.x, pos.y, 1.0),
        Visibility::default(),
        NodoLogico { tipo: TipoPuerta::Reloj },
    ))
    .with_children(|padre| {
        // 1. Recuadro izquierdo: Botón de frecuencia
        padre.spawn((
            Sprite {
                color: config.col_estado_0,
                custom_size: Some(Vec2::new(34.0, 26.0)),
                ..default()
            },
            Transform::from_xyz(-32.0, 0.0, 0.1),
            ControlReloj {
                estado: false,
                indice_frec: idx,
                acumulador: 0.0,
            },
        ))
        .with_children(|ctrl| {
            ctrl.spawn((
                Text2d::new(etiquetas_frec[idx]),
                TextFont {
                    font_size: bevy::text::FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 0.1),
                TextoReloj,
            ));
        });

        // 2. Cuadrado central: Cuerpo para arrastrar el reloj
        padre.spawn((
            Sprite {
                color: config.col_puerta,
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // 3. Triángulo derecho: Dirección hacia la salida
        padre.spawn((
            Mesh2d(malla_triangulo),
            MeshMaterial2d(mat_cuerpo),
            Transform::from_xyz(15.0, 0.0, 0.0),
        ));

        // 4. Conector de Salida en la punta del triángulo
        let e_out = padre.spawn(bundle_conector(
            Vec3::new(33.0, 0.0, 0.2),
            TipoConector::Salida,
            config,
        ))
        .with_children(|con| {
            con.spawn(bundle_hueco(config));
        })
        .id();
        ent_salida = Some(e_out);
    });

    (ent_salida, Vec::new())
}

// --- Bloque 7: Creador de Displays de 7 Segmentos ---
pub fn crear_display(
    commands: &mut Commands,
    pos: Vec2,
    modo_4_bits: bool,
    config: &ConfigApp,
) -> (Option<Entity>, Vec<Entity>) {
    let tipo = if modo_4_bits {
        TipoPuerta::Display4Bits
    } else {
        TipoPuerta::Display7Seg
    };

    let tam_cuerpo = Vec2::new(90.0, 130.0);
    let tam_pantalla = Vec2::new(56.0, 106.0);
    let mut entradas = Vec::new();

    commands.spawn((
        Sprite {
            color: config.col_puerta,
            custom_size: Some(tam_cuerpo),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 1.0),
        NodoLogico { tipo },
    ))
    .with_children(|padre| {
        // 1. Pantalla interior negra donde brillan los segmentos
        padre.spawn((
            Sprite {
                color: Color::srgb(0.06, 0.06, 0.08),
                custom_size: Some(tam_pantalla),
                ..default()
            },
            Transform::from_xyz(8.0, 0.0, 0.05),
        ));

        // 2. Conectores de Entrada en el borde izquierdo + etiquetas
        let num_entradas = if modo_4_bits { 4 } else { 7 };
        let etiquetas_4b = ["8", "4", "2", "1"];
        let etiquetas_7s = ["A", "B", "C", "D", "E", "F", "G"];

        for i in 0..num_entradas {
            let y = if modo_4_bits {
                36.0 - (i as f32 * 24.0)
            } else {
                54.0 - (i as f32 * 18.0)
            };

            let e_in = padre.spawn((
                bundle_conector(
                    Vec3::new(-tam_cuerpo.x / 2.0, y, 0.1),
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

            // Pequeña etiqueta junto al pin para saber qué bit o segmento es
            let txt_pin = if modo_4_bits {
                etiquetas_4b[i]
            } else {
                etiquetas_7s[i]
            };

            padre.spawn((
                Text2d::new(txt_pin),
                TextFont {
                    font_size: bevy::text::FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(-tam_cuerpo.x / 2.0 + 14.0, y, 0.2),
            ));
        }

        // 3. Los 7 segmentos geométricos (A, B, C, D, E, F, G)
        let cx = 8.0;
        let geom_segmentos: [(f32, f32, f32, f32); 7] = [
            (cx,        38.0, 28.0,  6.0), // 0: A (Arriba)
            (cx + 16.0, 19.0,  6.0, 30.0), // 1: B (Arriba-Derecha)
            (cx + 16.0,-19.0,  6.0, 30.0), // 2: C (Abajo-Derecha)
            (cx,       -38.0, 28.0,  6.0), // 3: D (Abajo)
            (cx - 16.0,-19.0,  6.0, 30.0), // 4: E (Abajo-Izquierda)
            (cx - 16.0, 19.0,  6.0, 30.0), // 5: F (Arriba-Izquierda)
            (cx,         0.0, 28.0,  6.0), // 6: G (Centro)
        ];

        let col_apagado = Color::srgb(0.18, 0.18, 0.20);
        for (idx, (sx, sy, sw, sh)) in geom_segmentos.iter().enumerate() {
            padre.spawn((
                Sprite {
                    color: col_apagado,
                    custom_size: Some(Vec2::new(*sw, *sh)),
                    ..default()
                },
                Transform::from_xyz(*sx, *sy, 0.15),
                SegmentoDisplay(idx),
            ));
        }
    });

    (None, entradas)
}

// --- Bloque 8: Creador de Barras Laterales para Chip ---
pub fn crear_barras_chip(commands: &mut Commands) {
    let x_izq = -500.0;
    let x_der = 500.0;
    let ancho_barra = 16.0;
    let alto_barra = 600.0;
    let col_barra = Color::srgb(0.22, 0.25, 0.30);

    // 1. Barra Izquierda (donde se pegan los Interruptores de Entrada)
    commands.spawn((
        Sprite {
            color: col_barra,
            custom_size: Some(Vec2::new(ancho_barra, alto_barra)),
            ..default()
        },
        Transform::from_xyz(x_izq, 0.0, 0.5),
        BarraChip::IzquierdaEntradas,
    ))
    .with_children(|padre| {
        padre.spawn((
            Text2d::new("IN CHIP"),
            TextFont {
                font_size: bevy::text::FontSize::Px(13.0),
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.8, 1.0)),
            Transform::from_xyz(0.0, alto_barra / 2.0 + 16.0, 0.1),
        ));
    });

    // 2. Barra Derecha (donde se pegan los LEDs de Salida)
    commands.spawn((
        Sprite {
            color: col_barra,
            custom_size: Some(Vec2::new(ancho_barra, alto_barra)),
            ..default()
        },
        Transform::from_xyz(x_der, 0.0, 0.5),
        BarraChip::DerechaSalidas,
    ))
    .with_children(|padre| {
        padre.spawn((
            Text2d::new("OUT CHIP"),
            TextFont {
                font_size: bevy::text::FontSize::Px(13.0),
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.8, 1.0)),
            Transform::from_xyz(0.0, alto_barra / 2.0 + 16.0, 0.1),
        ));
    });
}

// ==========================================
// Fichero: src/constructores.rs
// Bloque: Bloque 9: Creador de Chips Personalizados (NUEVO)
// ==========================================
pub fn crear_chip_personalizado(
    commands: &mut Commands,
    pos: Vec2,
    datos: DatosChip,
    config: &ConfigApp,
) -> (Vec<Entity>, Vec<Entity>) {
    let num_in = datos.pines_entrada.len();
    let num_out = datos.pines_salida.len();
    let max_pines = num_in.max(num_out).max(1);

    // Calculamos el tamaño de la carcasa según el número de patillas y largo del nombre
    let ancho_texto = (datos.nombre.len() as f32 * 11.0).max(80.0);
    let ancho_cuerpo = ancho_texto + 28.0;
    let separacion_pines = 24.0;
    let alto_cuerpo = (max_pines as f32 * separacion_pines + 16.0).max(50.0);

    // Inicializamos la memoria de estados de cada nodo interno del chip
    let mut estados_nodos = Vec::with_capacity(datos.nodos.len());
    for n in datos.nodos.iter() {
        let num_salidas = if let Some(sub) = &n.sub_chip {
            sub.pines_salida.len().max(1)
        } else {
            1
        };
        estados_nodos.push(vec![false; num_salidas]);
    }

    let nombre_mostrar = datos.nombre.clone();
    let mut salidas_ent = Vec::new();
    let mut entradas_ent = Vec::new();

    commands.spawn((
        Sprite {
            color: Color::srgb(0.18, 0.32, 0.48),
            custom_size: Some(Vec2::new(ancho_cuerpo, alto_cuerpo)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 1.0),
        NodoLogico { tipo: TipoPuerta::Chip },
        InstanciaChip {
            datos,
            estados_nodos,
        },
    ))
    .with_children(|padre| {
        // Texto con el nombre del Chip en el centro
        padre.spawn((
            Text2d::new(nombre_mostrar),
            TextFont {
                font_size: bevy::text::FontSize::Px(16.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(0.0, 0.0, 0.2),
        ));

        // Conectores de Entrada en el lado izquierdo (ordenados de arriba a abajo)
        let inicio_in_y = 
            ((num_in.saturating_sub(1)) as f32 * separacion_pines) / 2.0;
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
                con.spawn(bundle_hueco(config));
            })
            .id();
            entradas_ent.push(e_in);
        }

        // Conectores de Salida en el lado derecho (ordenados de arriba a abajo)
        let inicio_out_y = 
            ((num_out.saturating_sub(1)) as f32 * separacion_pines) / 2.0;
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
                con.spawn(bundle_hueco(config));
            })
            .id();
            salidas_ent.push(e_out);
        }
    });

    (salidas_ent, entradas_ent)
}