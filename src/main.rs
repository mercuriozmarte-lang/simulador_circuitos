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
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Simulador Lógico Moderno".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        // Inyectamos la configuración, recursos de ratón y pila de inspección
        .init_resource::<ConfigApp>()
        .init_resource::<EstadoCrearLinea>()
        .init_resource::<CodoArrastrado>()
        .init_resource::<PilaInspeccion>()
        .init_state::<ModoApp>()
        .add_systems(Startup, setup_entorno)
        .add_systems(
            Update, 
            (
                (
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
    q_barra_inf: Query<Entity, With<BarraInferiorChips>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<ConfigApp>,
) {
    commands.spawn((
        Camera2d, 
        CamaraPrincipal
    ));

    // Creamos la cuadrícula fija en el fondo (Z = -5.0) detrás de todo
    crear_cuadricula_fondo(&mut commands, &config);

    // Creamos la barra inferior flotante cargando los Chips (.cir) que existan en disco
    reconstruir_barra_inferior_chips(&mut commands, &q_barra_inf);

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

    // Instanciamos una puerta AND, una OR y una NOT vectoriales
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

    // Instanciamos una puerta NOT de una sola entrada
    crear_puerta(
        &mut commands,
        &mut meshes,
        &mut materials,
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