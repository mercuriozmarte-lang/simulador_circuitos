// ==========================================
// Fichero: src/configuracion.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;

// Bloque: Bloque 2: Recurso de Configuración
#[derive(Resource)]
pub struct ConfigApp {
    pub col_cuadricula: Color,
    pub col_puerta: Color,
    pub col_con_ent: Color,
    pub col_con_sal: Color,
    pub col_cable_tmp: Color,
    pub col_cable_fij: Color,
    // Colores de estado lógico (0, 1 y desconectado)
    pub col_estado_0: Color,
    pub col_estado_1: Color,
    pub col_con_hueco: Color,
    pub tam_puerta: Vec2,
    pub tam_conector: Vec2,
    pub rad_clic_nodo: f32,
    pub rad_clic_con: f32,
}

// Bloque: Bloque 3: Valores por Defecto
impl Default for ConfigApp {
    fn default() -> Self {
        Self {
            col_cuadricula: Color::srgb(0.2, 0.2, 0.2),
            col_puerta: Color::srgb(0.2, 0.6, 0.8),
            col_con_ent: Color::srgb(0.2, 0.8, 0.2),
            col_con_sal: Color::srgb(0.8, 0.2, 0.2),
            col_cable_tmp: Color::WHITE,
            col_cable_fij: Color::srgb(0.0, 1.0, 1.0),
            // Rojo para 0, Verde para 1, Gris para borde desconectado
            col_estado_0: Color::srgb(0.85, 0.15, 0.15),
            col_estado_1: Color::srgb(0.15, 0.85, 0.15),
            col_con_hueco: Color::srgb(0.45, 0.45, 0.45),
            tam_puerta: Vec2::new(80.0, 50.0),
            tam_conector: Vec2::new(12.0, 12.0),
            rad_clic_nodo: 40.0,
            rad_clic_con: 15.0,
        }
    }
}
