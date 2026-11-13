// ==========================================
// Fichero: src/main.rs
// ==========================================

// --- Bloque 1: Importaciones y Módulos ---
use bevy::prelude::*;

pub mod configuracion;
pub mod componentes;
pub mod raton;
pub mod lineas;
pub mod logica;
pub mod archivo;

use configuracion::ConfigApp;
use componentes::*;
use raton::*;
use lineas::*;
use logica::*;
use archivo::*;

// --- Bloque 2: main ---
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Simulador Lógico Moderno".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        // Inyectamos la configuración
        .init_resource::<ConfigApp>()
        .init_resource::<EstadoCrearLinea>()
        .init_state::<ModoApp>()
        .add_systems(Startup, setup_entorno)
        .add_systems(
            Update, 
            (
                (
                    dibuja_cuadricula,
                    gestionar_clic_menu,
                    detectar_clic_nodo,
                    soltar_nodo,
                    mover_nodo_arrastrado,
                    borrar_con_clic_derecho,
                    procesar_guardado_y_carga,
                ).chain(),
                (
                    limpiar_cables_huerfanos,
                    // Temporizador de relojes y cerebro lógico
                    actualizar_relojes,
                    propagar_cables,
                    evaluar_puertas,
                    actualizar_visuales_conectores,
                    // Dibujado de cables
                    dibuja_linea_temporal,
                    dibuja_cables_permanentes,
                ).chain(),
            ).chain()
        )
        .run();
}

// --- Bloque 3: Setup del Entorno ---
fn setup_entorno(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<ConfigApp>,
) {
    commands.spawn((
        Camera2d, 
        CamaraPrincipal
    ));

    // Creamos dos interruptores de entrada a la izquierda
    crear_interruptor(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(-300.0, 50.0),
        false,
        &config,
    );
    crear_interruptor(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(-300.0, -50.0),
        false,
        &config,
    );

    // Instanciamos una puerta AND, una OR y una NOT
    crear_puerta(
        &mut commands, 
        Vec2::new(-50.0, 50.0), 
        TipoPuerta::And,
        &config
    );
    
    crear_puerta(
        &mut commands, 
        Vec2::new(150.0, 50.0), 
        TipoPuerta::Or,
        &config
    );

    // Instanciamos una puerta NOT de una sola entrada
    crear_puerta(
        &mut commands, 
        Vec2::new(-50.0, -100.0), 
        TipoPuerta::Not,
        &config
    );

    // Creamos un LED de salida a la derecha para cerrar el circuito
    crear_led(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(350.0, 0.0),
        &config,
    );
}

// --- Bloque 4: Creador de Puertas y Conectores ---
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

pub fn crear_puerta(
    commands: &mut Commands, 
    pos: Vec2,
    tipo_puerta: TipoPuerta,
    config: &ConfigApp,
) -> (Option<Entity>, Vec<Entity>) {
    let texto = match tipo_puerta {
        TipoPuerta::And => "AND",
        TipoPuerta::Or => "OR",
        TipoPuerta::Not => "NOT",
        TipoPuerta::Interruptor => "IN",
        TipoPuerta::Led => "OUT",
        TipoPuerta::Reloj => "CLK",
        TipoPuerta::Display7Seg => "7SEG",
        TipoPuerta::Display4Bits => "HEX",
    };

    let mut ent_salida = None;
    let mut entradas = Vec::new();

    commands.spawn((
        Sprite {
            color: config.col_puerta,
            custom_size: Some(config.tam_puerta),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 1.0),
        NodoLogico { tipo: tipo_puerta },
    ))
    .with_children(|padre| {
        padre.spawn((
            Text2d::new(texto),
            TextFont {
                font_size: bevy::text::FontSize::Px(20.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(0.0, 0.0, 0.2),
        ));
        
        if tipo_puerta == TipoPuerta::Not {
            let e_in = padre.spawn((
                bundle_conector(
                    Vec3::new(-config.tam_puerta.x / 2.0, 0.0, 0.1),
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
                    Vec3::new(-config.tam_puerta.x / 2.0, 15.0, 0.1),
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
                    Vec3::new(-config.tam_puerta.x / 2.0, -15.0, 0.1),
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
            Vec3::new(config.tam_puerta.x / 2.0, 0.0, 0.1),
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

// --- Bloque 5: Creador de Interruptores ---
pub fn crear_interruptor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec2,
    estado_inicial: bool,
    config: &ConfigApp,
) -> (Option<Entity>, Vec<Entity>) {
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
        padre.spawn((
            Sprite {
                color: col_btn,
                custom_size: Some(Vec2::new(26.0, 26.0)),
                ..default()
            },
            Transform::from_xyz(-28.0, 0.0, 0.1),
            BotonInterruptor { estado: estado_inicial },
        ));

        padre.spawn((
            Sprite {
                color: config.col_puerta,
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        padre.spawn((
            Mesh2d(malla_triangulo),
            MeshMaterial2d(mat_cuerpo),
            Transform::from_xyz(15.0, 0.0, 0.0),
        ));

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

// --- Bloque 6: Creador de LEDs de Salida ---
pub fn crear_led(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec2,
    config: &ConfigApp,
) -> (Option<Entity>, Vec<Entity>) {
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

        padre.spawn((
            Mesh2d(malla_triangulo),
            MeshMaterial2d(mat_cuerpo),
            Transform::from_xyz(-15.0, 0.0, 0.0),
        ));

        padre.spawn((
            Sprite {
                color: config.col_puerta,
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

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

// --- Bloque 7: Creador de Relojes (Clock) ---
pub fn crear_reloj(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec2,
    indice_frec_inicial: usize,
    config: &ConfigApp,
) -> (Option<Entity>, Vec<Entity>) {
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

        padre.spawn((
            Sprite {
                color: config.col_puerta,
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        padre.spawn((
            Mesh2d(malla_triangulo),
            MeshMaterial2d(mat_cuerpo),
            Transform::from_xyz(15.0, 0.0, 0.0),
        ));

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

// --- Bloque 8: Creador de Displays de 7 Segmentos ---
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
        padre.spawn((
            Sprite {
                color: Color::srgb(0.06, 0.06, 0.08),
                custom_size: Some(tam_pantalla),
                ..default()
            },
            Transform::from_xyz(8.0, 0.0, 0.05),
        ));

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