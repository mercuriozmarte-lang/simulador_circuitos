// ==========================================
// Fichero: src/configuracion.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;

// ==========================================
// Fichero: src/configuracion.rs
// Bloque: Bloque 2: Estructura de Configuración Global
// ==========================================
#[derive(Resource)]
pub struct ConfigApp {
    // Dimensiones geométricas
    pub tam_puerta: Vec2,
    pub tam_conector: Vec2,
    pub grosor_cable: f32,
    // Separación vertical entre patillas de un Chip (25.0 encaja con cuadrícula y snap de 5px)
    pub sep_pines_chip: f32,
    // Dimensiones del cuerpo de agarre de Interruptores y LEDs
    pub ancho_agarre_io: f32,
    pub alto_io: f32,
    
    // Radios de detección y pasos de ajuste (Snap)
    pub rad_clic_con: f32,
    pub rad_clic_nodo: f32,
    pub paso_snap_puerta: f32,
    pub paso_snap_io: f32,
    pub paso_snap_codo: f32,

    // Paleta de colores del lienzo, barras y puertas
    pub col_fondo: Color,
    pub col_cuadricula: Color,
    pub col_barra_io: Color,
    pub col_puerta: Color,
    pub col_patilla: Color,
    pub col_con_hueco: Color,
    pub col_cable_tmp: Color,

    // Colores para las etiquetas de nombre de Pin (Fondo negro y letra verde claro)
    pub col_etiqueta_fondo: Color,
    pub col_etiqueta_texto: Color,
    
    // Colores de estado lógico (0 y 1)
    pub col_estado_0: Color,
    pub col_estado_1: Color,
}

// ==========================================
// Fichero: src/configuracion.rs
// Bloque: Bloque 3: Valores por Defecto
// ==========================================
impl Default for ConfigApp {
    fn default() -> Self {
        Self {
            tam_puerta: Vec2::new(100.0, 60.0),
            tam_conector: Vec2::new(12.0, 12.0),
            grosor_cable: 3.5,
            // Paso entre pines de Chip: 25.0px (múltiplo exacto de paso_snap_io = 5.0)
            sep_pines_chip: 25.0,
            // Cuerpo de agarre más ancho (38px) y más bajo (20px) para Interruptores y LEDs
            ancho_agarre_io: 38.0,
            alto_io: 20.0,
            
            rad_clic_con: 12.0,
            rad_clic_nodo: 35.0,
            paso_snap_puerta: 50.0,
            paso_snap_io: 5.0,
            paso_snap_codo: 5.0,

            col_fondo: Color::srgb(0.90, 0.92, 0.96),
            col_cuadricula: Color::srgb(0.76, 0.79, 0.84),
            col_barra_io: Color::srgb(0.75, 0.78, 0.83),
            col_puerta: Color::srgb(0.35, 0.38, 0.44),
            col_patilla: Color::srgb(0.08, 0.08, 0.10),
            col_con_hueco: Color::srgb(0.60, 0.62, 0.66),
            col_cable_tmp: Color::srgb(0.85, 0.55, 0.0),

            col_etiqueta_fondo: Color::srgb(0.05, 0.06, 0.08),
            col_etiqueta_texto: Color::srgb(0.25, 1.0, 0.35),
            
            col_estado_0: Color::srgb(0.85, 0.20, 0.20),
            col_estado_1: Color::srgb(0.0, 0.6, 0.0),
        }
    }
}