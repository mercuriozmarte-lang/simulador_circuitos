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

// Fichero: src/componentes.rs
// Bloque: Bloque 4: Componentes de Nodos
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TipoPuerta {
    And,
    Or,
    Not,
}

// Ahora el nodo recuerda qué operación matemática debe hacer
#[derive(Component)]
pub struct NodoLogico {
    pub tipo: TipoPuerta,
}

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