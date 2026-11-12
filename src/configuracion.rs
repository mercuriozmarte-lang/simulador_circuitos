// ==========================================
// Fichero: src/configuracion.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;

// --- Bloque 2: Recurso de Configuración ---
#[derive(Resource)]
pub struct ConfigApp {
    pub col_cuadricula: Color,
    pub col_puerta: Color,
    pub col_con_ent: Color,
    pub col_con_sal: Color,
    pub col_cable_tmp: Color,
    pub col_cable_fij: Color,
    pub tam_puerta: Vec2,
    pub tam_conector: Vec2,
    pub rad_clic_nodo: f32,
    pub rad_clic_con: f32,
}

// --- Bloque 3: Valores por Defecto ---
impl Default for ConfigApp {
    fn default() -> Self {
        Self {
            col_cuadricula: Color::srgb(0.2, 0.2, 0.2),
            col_puerta: Color::srgb(0.2, 0.6, 0.8),
            col_con_ent: Color::srgb(0.2, 0.8, 0.2),
            col_con_sal: Color::srgb(0.8, 0.2, 0.2),
            col_cable_tmp: Color::WHITE,
            col_cable_fij: Color::srgb(0.0, 1.0, 1.0),
            tam_puerta: Vec2::new(80.0, 50.0),
            tam_conector: Vec2::new(12.0, 12.0),
            rad_clic_nodo: 40.0,
            rad_clic_con: 15.0,
        }
    }
}