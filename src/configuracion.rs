// ==========================================
// Fichero: src/configuracion.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;

// --- Bloque 2: Estructura de Configuración ---
#[derive(Resource)]
pub struct ConfigApp {
    // Dimensiones geométricas
    pub tam_puerta: Vec2,
    pub tam_conector: Vec2,
    // NUEVO: Grosor en píxeles de los hilos (cables)
    pub grosor_cable: f32,
    
    // Radios de detección del ratón
    pub rad_clic_con: f32,
    pub rad_clic_nodo: f32,
    // Paso de ajuste (snap) para los codos de los cables (5.0 permite alinear con Y=+-15)
    pub paso_snap_codo: f32,

    // Paleta de colores del fondo y puertas
    pub col_fondo: Color,
    pub col_cuadricula: Color,
    pub col_puerta: Color,
    pub col_patilla: Color,
    pub col_con_hueco: Color,
    pub col_cable_tmp: Color,
    
    // Colores de estado lógico
    pub col_estado_0: Color,
    pub col_estado_1: Color,
}

// --- Bloque 3: Valores por Defecto ---
impl Default for ConfigApp {
    fn default() -> Self {
        Self {
            tam_puerta: Vec2::new(100.0, 60.0),
            tam_conector: Vec2::new(12.0, 12.0),
            // Grosor de los cables (prueba entre 2.5 y 5.0 según te guste más)
            grosor_cable: 3.5,
            
            rad_clic_con: 12.0,
            rad_clic_nodo: 35.0,
            // Con 5.0 los codos encajan exactos tanto en la rejilla (50) como en patillas (+-15)
            paso_snap_codo: 5.0,

            // Fondo claro elegido por ti
            col_fondo: Color::srgb(0.90, 0.92, 0.96),
            // Cuadrícula suave para que no compita visualmente con los cables
            col_cuadricula: Color::srgb(0.76, 0.79, 0.84),
            col_puerta: Color::srgb(0.35, 0.38, 0.44),
            // Color de las patillas y bordes de puertas
            col_patilla: Color::srgb(0.08, 0.08, 0.10),
            col_con_hueco: Color::srgb(0.60, 0.62, 0.66),
            col_cable_tmp: Color::srgb(0.85, 0.55, 0.0),
            
            // Rojo para 0, Verde elegido por ti para 1
            col_estado_0: Color::srgb(0.85, 0.20, 0.20),
            col_estado_1: Color::srgb(0.0, 0.6, 0.0),
        }
    }
}