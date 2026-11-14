// ==========================================
// Fichero: src/main.rs
// ==========================================

// --- Bloque 1: Importaciones y Módulos ---
use bevy::prelude::*;

pub mod configuracion;
pub mod componentes;
pub mod constructores;
pub mod raton;
pub mod lineas;
pub mod logica;
pub mod archivo;

use configuracion::ConfigApp;
use componentes::*;
pub use constructores::*;
use raton::*;
use lineas::*;
use logica::*;
use archivo::*;
// ==========================================
// Fichero: src/main.rs
// Bloque: Bloque 2: main
// ==========================================
fn main() {
    let config_inicial = ConfigApp::default();
    let color_fondo = config_inicial.col_fondo;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Simulador Lógico Moderno".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        // Aplicamos el color de fondo configurado en ConfigApp
        .insert_resource(ClearColor(color_fondo))
        .insert_resource(config_inicial)
        .init_resource::<EstadoCrearLinea>()
        .init_resource::<CodoArrastrado>()
        .init_resource::<PilaInspeccion>()
        .init_resource::<HistorialUndo>()
        .init_state::<ModoApp>()
        .add_systems(Startup, setup_entorno)
        .add_systems(
            Update, 
            (
                (
                    controlar_camara,
                    actualizar_cuadricula_infinita,
                    gestionar_teclado_archivo,
                    gestionar_clic_selector_carga,
                    gestionar_clic_barra_inferior,
                    gestionar_clic_menu,
                    detectar_clic_nodo,
                    soltar_nodo,
                    mover_nodo_arrastrado,
                    borrar_con_clic_derecho,
                    procesar_guardado_y_carga,
                ).chain(),
                (
                    limpiar_cables_huerfanos,
                    actualizar_relojes,
                    propagar_cables,
                    evaluar_puertas,
                    actualizar_visuales_conectores,
                    dibuja_linea_temporal,
                    dibuja_cables_permanentes,
                ).chain(),
            ).chain()
        )
        .run();
}

// ==========================================
// Fichero: src/main.rs
// Bloque: Bloque 3: Setup del Entorno
// ==========================================
fn setup_entorno(
    mut commands: Commands,
    q_barra_inf: Query<Entity, With<BarraInferiorChips>>,
    mut config_store: ResMut<GizmoConfigStore>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<ConfigApp>,
) {
    commands.spawn((
        Camera2d, 
        CamaraPrincipal
    ));

    // Configuramos el grosor de los cables y creamos la cuadrícula en el fondo
    crear_cuadricula_fondo(
        &mut commands, 
        &mut config_store, 
        &config
    );
    reconstruir_barra_inferior_chips(&mut commands, &q_barra_inf);

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

    crear_puerta(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(-50.0, 50.0), 
        TipoPuerta::And,
        &config
    );
    
    crear_puerta(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(150.0, 50.0), 
        TipoPuerta::Or,
        &config
    );

    crear_puerta(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(-50.0, -100.0), 
        TipoPuerta::Not,
        &config
    );

    crear_led(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(350.0, 0.0),
        &config,
    );
}
