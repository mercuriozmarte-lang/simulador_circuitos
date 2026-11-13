// ==========================================
// Fichero: src/componentes.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;

// --- Bloque 2: Estados ---
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum ModoApp {
    #[default]
    Edicion,
    CreandoLinea,
}

// --- Bloque 3: Recursos ---
#[derive(Resource, Default)]
pub struct EstadoCrearLinea {
    pub origen_entidad: Option<Entity>,
    pub origen_pos: Vec2,
}

// ==========================================
// Fichero: src/componentes.rs
// Bloque: Bloque 4: Componentes de Nodos
// ==========================================
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TipoPuerta {
    And,
    Or,
    Not,
    // Interruptor de entrada manual
    Interruptor,
    // Elemento receptor final (LED / Bombilla)
    Led,
    // NUEVO: Generador de pulsos de reloj
    Reloj,
}

// Ahora el nodo recuerda qué operación matemática debe hacer
#[derive(Component)]
pub struct NodoLogico {
    pub tipo: TipoPuerta,
}

// Identifica el cuadrado izquierdo que alterna entre 0 y 1
#[derive(Component)]
pub struct BotonInterruptor {
    pub estado: bool,
}

// NUEVO: Identifica el cuadrado izquierdo del reloj que controla su frecuencia
#[derive(Component)]
pub struct ControlReloj {
    pub estado: bool,
    pub indice_frec: usize,
    pub acumulador: f32,
}

// NUEVO: Identifica el texto que muestra los Hz dentro del reloj
#[derive(Component)]
pub struct TextoReloj;

// Identifica el bloque luminoso del LED de salida
#[derive(Component)]
pub struct IndicadorLed;

// Identifica el interior oscuro del conector para verse "hueco"
#[derive(Component)]
pub struct CentroHueco;

#[derive(Component)]
pub struct Arrastrando;

#[derive(Component)]
pub struct CamaraPrincipal;

// --- Bloque 5: Componentes de Conexión ---
#[derive(Component, PartialEq, Eq)]
pub enum TipoConector {
    Entrada,
    Salida,
}

#[derive(Component)]
pub struct Cable {
    pub origen: Entity,
    pub destino: Entity,
}

// Bloque: Bloque 6: Estado lógico de los conectores (NUEVO)
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ValorSeñal(pub bool);

// Bloque: Bloque 7: Componentes del Menú Contextual
#[derive(Component)]
pub struct MenuContextual {
    // Guardamos dónde se hizo clic derecho para crear ahí la pieza
    pub pos_creacion: Vec2,
}

#[derive(Component)]
pub struct BotonMenu {
    pub tipo: TipoPuerta,
}
