// ==========================================
// Fichero: src/componentes.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;

// --- Bloque 2: Estados de la Aplicación ---
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModoApp {
    #[default]
    Edicion,
    CreandoLinea,
}

// ==========================================
// Fichero: src/componentes.rs
// Bloque: Bloque 3: Recursos Globales
// ==========================================
#[derive(Resource, Default)]
pub struct EstadoCrearLinea {
    pub origen_entidad: Option<Entity>,
    pub origen_pos: Vec2,
    pub puntos_intermedios: Vec<Vec2>,
}

#[derive(Resource, Default)]
pub struct CodoArrastrado {
    pub seleccion: Option<(Entity, usize)>,
}

#[derive(Resource, Default)]
pub struct PilaInspeccion {
    pub niveles: Vec<(String, String)>,
}

#[derive(Resource, Default)]
pub struct HistorialUndo {
    pub estados: Vec<String>,
}

#[derive(Resource, Default)]
pub struct DesplazamientoBarraChips {
    pub indice_inicio: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ModoAjusteBarra {
    MoverHorizontal,
    CambiarAltura,
}

#[derive(Resource, Default)]
pub struct EstadoArrastreBarra {
    pub activo: Option<(BarraChip, ModoAjusteBarra)>,
}

// NUEVO: Controla el rectángulo de selección múltiple con arrastre de ratón izquierdo
#[derive(Resource, Default)]
pub struct CajaSeleccion {
    pub activa: bool,
    pub inicio: Vec2,
    pub actual: Vec2,
}

// ==========================================
// Fichero: src/componentes.rs
// Bloque: Bloque 4: Componentes de Nodos, Etiquetas y Chips
// ==========================================
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TipoPuerta {
    And,
    Or,
    Not,
    Nand,
    Nor,
    Xor,
    Xnor,
    TriState,
    Interruptor,
    Led,
    Reloj,
    Display7Seg,
    Display4Bits,
    Chip,
}

#[derive(Component)]
pub struct NodoLogico {
    pub tipo: TipoPuerta,
}

#[derive(Clone, Debug)]
pub struct NodoInternoChip {
    pub cod_tipo: String,
    pub sub_chip: Option< Box<DatosChip> >,
}

#[derive(Clone, Debug, Default)]
pub struct DatosChip {
    pub nombre: String,
    pub nodos: Vec<NodoInternoChip>,
    pub cables: Vec<(usize, usize, usize, usize)>,
    pub pines_entrada: Vec<usize>,
    pub nombres_entrada: Vec<String>,
    pub pines_salida: Vec<usize>,
    pub nombres_salida: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct EstadoRedChip {
    pub salidas_nodos: Vec< Vec<bool> >,
    pub sub_redes: Vec< Option<Box<EstadoRedChip>> >,
}

impl EstadoRedChip {
    pub fn nuevo_desde_datos(datos: &DatosChip) -> Self {
        let mut salidas_nodos = Vec::with_capacity(datos.nodos.len());
        let mut sub_redes = Vec::with_capacity(datos.nodos.len());

        for n in datos.nodos.iter() {
            if let Some(sub) = &n.sub_chip {
                let n_out = sub.pines_salida.len().max(1);
                salidas_nodos.push(vec![false; n_out]);
                sub_redes.push(Some(Box::new(Self::nuevo_desde_datos(sub))));
            } else {
                salidas_nodos.push(vec![false; 1]);
                sub_redes.push(None);
            }
        }

        Self {
            salidas_nodos,
            sub_redes,
        }
    }
}

#[derive(Component, Clone)]
pub struct InstanciaChip {
    pub datos: DatosChip,
    pub memoria: EstadoRedChip,
}

#[derive(Component, Clone, Default)]
pub struct EtiquetaNodo(pub String);

#[derive(Component)]
pub struct FondoEtiquetaNodo;

#[derive(Component)]
pub struct TextoEtiquetaNodo;

#[derive(Component)]
pub struct BotonVerInteriorChip {
    pub archivo_chp: String,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum BarraChip {
    IzquierdaEntradas,
    DerechaSalidas,
}

#[derive(Component)]
pub struct PegadoABarra(pub BarraChip);

// ==========================================
// Fichero: src/componentes.rs
// Bloque: Bloque 5: Componentes Internos de Control Visual
// ==========================================
#[derive(Component)]
pub struct BotonInterruptor {
    pub estado: bool,
}

#[derive(Component)]
pub struct ControlReloj {
    pub estado: bool,
    pub indice_frec: usize,
    pub acumulador: f32,
}

#[derive(Component)]
pub struct TextoReloj;

#[derive(Component)]
pub struct IndicadorLed;

#[derive(Component)]
pub struct PinEntrada(pub usize);

#[derive(Component)]
pub struct PinSalida(pub usize);

#[derive(Component)]
pub struct SegmentoDisplay(pub usize);

#[derive(Component)]
pub struct CentroHueco;

// Guarda el desplazamiento X e Y respecto al cursor para mover columnas o bloques seleccionados
#[derive(Component)]
pub struct Arrastrando {
    pub offset: Vec2,
}

// NUEVO: Marca los nodos que están dentro de la selección actual
#[derive(Component)]
pub struct Seleccionado;

#[derive(Component)]
pub struct AsaAltoBarra(pub BarraChip);

#[derive(Component)]
pub struct AsaMoverBarra(pub BarraChip);

#[derive(Component)]
pub struct EtiquetaSuperiorBarra;

#[derive(Component)]
pub struct CamaraPrincipal;

#[derive(Component)]
pub struct CuadriculaFondo;

// ==========================================
// Fichero: src/componentes.rs
// --- Bloque 6: Componentes de Conectores y Cables ---
// ==========================================
#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum TipoConector {
    Entrada,
    Salida,
}

#[derive(Component)]
pub struct ValorSeñal(pub bool);

#[derive(Component)]
pub struct Cable {
    pub origen: Entity,
    pub destino: Entity,
    pub puntos_intermedios: Vec<Vec2>,
}

// ==========================================
// Fichero: src/componentes.rs
// Bloque: Bloque 7: Componentes de Menús, Diálogos y Barras UI
// ==========================================
#[derive(Component)]
pub struct MenuContextual {
    pub pos_creacion: Vec2,
}

#[derive(Component)]
pub struct SubMenuContextual {
    pub categoria: CategoriaMenu,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CategoriaMenu {
    Puertas,
    Entradas,
    Salidas,
    Chip,
    Archivo,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccionMenu {
    AbrirCategoria(CategoriaMenu),
    CrearNodo(TipoPuerta),
    AlternarBarrasChip,
    GuardarChip,
    GuardarCircuito,
    CargarCircuito,
    LimpiarTodo,
}

#[derive(Component)]
pub struct BotonMenu {
    pub accion: AccionMenu,
    pub centro_mundo: Vec2,
}

#[derive(Component)]
pub struct DialogoArchivo {
    pub es_chip: bool,
    pub nombre: String,
}

#[derive(Component)]
pub struct TextoInputArchivo;

#[derive(Component)]
pub struct DialogoEtiquetaPin {
    pub nodo_objetivo: Entity,
    pub texto: String,
}

#[derive(Component)]
pub struct TextoInputEtiqueta;

#[derive(Component)]
pub struct SelectorCarga;

#[derive(Component)]
pub struct BotonArchivoCir {
    pub nombre_archivo: String,
    pub centro_mundo: Vec2,
}

#[derive(Component)]
pub struct BarraInferiorChips;

#[derive(Component)]
pub struct BotonChipInferior {
    pub archivo_chp: String,
    pub x_min: f32,
    pub x_max: f32,
}

// NUEVO: Botones [<] y [>] para deslizar la barra inferior de Chips
#[derive(Component)]
pub struct BotonScrollBarra {
    pub delta: isize,
    pub x_min: f32,
    pub x_max: f32,
}

#[derive(Component)]
pub struct BarraSuperiorInspeccion;

#[derive(Component)]
pub struct BotonVolverInspeccion;