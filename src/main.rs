// ==========================================
// Fichero: src/main.rs
// Bloque: Bloque 1: Manual de Instrucciones y Controles del Simulador
// ==========================================
// ============================================================================
//                 MANUAL DE CONTROLES - SIMULADOR LÓGICO 2D
// ============================================================================
// 1. NAVEGACIÓN POR EL LIENZO INFINITO (PAN & ZOOM):
//    - Rueda del ratón (Scroll)       : Acercar / Alejar Zoom (0.25x a 4.0x).
//    - Botón central (Rueda pulsada)  : Arrastrar el lienzo (Pan).
//    - Espacio + Clic Izquierdo       : Arrastrar el lienzo (alternativa).
//    - Flechas Izquierda / Derecha    : Desplazar la cámara horizontalmente.
//    - Tecla Inicio (Home)            : Volver al centro (0, 0) con Zoom 1.0x.
//
// 2. CREACIÓN, APILADO MÚLTIPLE (ESTILO DLS) Y EDICIÓN DE COMPONENTES:
//    - Clic en Chip de barra inferior : El Chip aparece pegado al cursor.
//        * Mientras lo llevas pegado al ratón:
//          - Pulsar [ D ] o [Flecha Arriba] : Añade otra copia a la columna.
//          - Pulsar [Flecha Abajo]          : Quita una copia de la columna.
//          - Clic Izquierdo en el lienzo    : Suelta toda la columna de golpe.
//          - Clic Derecho o [ Esc ]         : Cancela la colocación.
//        * Atajo rápido desde la barra inferior:
//          - Ctrl + Clic en botón           : Engancha 2 copias al cursor.
//          - Shift + Clic en botón          : Engancha 4 copias al cursor.
//    - Tecla [ D ] sobre pieza fija   : Duplica la pieza bajo el cursor.
//    - Clic Izquierdo sobre pieza     : Arrastrar componente.
//        * Puertas / Chips            : Hacen snap a la cuadrícula (50px).
//        * Interruptores / LEDs       : Hacen snap fino (5px) e imantan a barras.
//
// 3. TRAZADO DE CABLES Y CODOS (MODO 90 GRADOS):
//    - Clic Izquierdo en Salida (der) : Iniciar el trazado de un nuevo cable.
//    - Clic Izquierdo en el lienzo    : Añadir una esquina (codo) al cable.
//    - Mantener Tecla [ L ]           : Forzar línea horizontal o vertical (90°).
//    - Clic Izquierdo en Entrada (izq): Conectar y finalizar el cable.
//    - Arrastrar un codo con Clic Izq : Mover la esquina del cable (snap de 5px).
//    - Clic Derecho sobre un codo     : Eliminar solo esa esquina del cable.
//    - Clic Derecho sobre tramo recto : Eliminar el cable completo.
//
// 4. NOMBRADO DE CONEXIONES / PINES:
//    - Tecla [ N ] sobre Interruptor  : Abrir cuadro para escribir el nombre del
//      o sobre un LED                   pin (ej: A, B, CIN, SUM).
//
// 5. CARPETAS DE ARCHIVOS (circuitos/ y chips/) E INSPECCIÓN:
//    - Carpeta "chips/"               : Guarda y carga todos los chips (.chp).
//    - Carpeta "circuitos/"           : Guarda y carga los circuitos (.cir).
//    - Botón [ VER ] dentro de un Chip: Entra a inspeccionar/editar el interior.
//
// 6. HISTORIAL (DESHACER):
//    - Ctrl + Z                       : Deshacer la última modificación (Undo).
// ============================================================================

// --- Bloque 2: Declaración de Módulos e Importaciones ---
use bevy::prelude::*;

pub mod configuracion;
pub mod componentes;
pub mod constructores;
pub mod camara;
pub mod menus;
pub mod raton;
pub mod lineas;
pub mod logica;
pub mod archivo;

use configuracion::ConfigApp;
use componentes::*;
pub use constructores::*;
use camara::*;
use menus::*;
use raton::*;
use lineas::*;
use logica::*;
use archivo::*;

// ==========================================
// Fichero: src/main.rs
// Bloque: Bloque 3: Función Principal (main)
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
        .insert_resource(ClearColor(color_fondo))
        .insert_resource(config_inicial)
        .init_resource::<EstadoCrearLinea>()
        .init_resource::<CodoArrastrado>()
        .init_resource::<PilaInspeccion>()
        .init_resource::<HistorialUndo>()
        .init_resource::<DesplazamientoBarraChips>()
        .init_state::<ModoApp>()
        .add_systems(Startup, setup_entorno)
        .add_systems(
            Update, 
            (
                (
                    controlar_camara,
                    actualizar_cuadricula_infinita,
                    gestionar_teclado_archivo,
                    gestionar_nombrado_pin,
                    gestionar_clic_selector_carga,
                    gestionar_clic_barras_ui,
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
// Bloque: Bloque 4: Inicialización del Entorno (setup_entorno)
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

    crear_cuadricula_fondo(
        &mut commands, 
        &mut config_store, 
        &config
    );
    reconstruir_barra_inferior_chips(&mut commands, &q_barra_inf, 0);

    crear_interruptor(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(-300.0, 50.0),
        false,
        "A",
        &config,
    );
    crear_interruptor(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(-300.0, -50.0),
        false,
        "B",
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
        "OUT",
        &config,
    );
}