// ==========================================
// Fichero: src/componentes.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;

// --- Bloque 2: Estados de la Aplicación ---
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ModoApp {
    #[default]
    Edicion,
    CreandoLinea,
}


// Bloque: Bloque 3: Recursos Globales
#[derive(Resource, Default)]
pub struct EstadoCrearLinea {
    pub origen_entidad: Option<Entity>,
    pub origen_pos: Vec2,
    // Almacena los codos (waypoints) mientras trazamos el cable
    pub puntos_intermedios: Vec<Vec2>,
}

// Guarda qué codo de qué cable estamos arrastrando con el ratón
#[derive(Resource, Default)]
pub struct CodoArrastrado {
    pub seleccion: Option<(Entity, usize)>,
}

// Bloque: Bloque 4: Componentes de Nodos y Barras de Chip
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TipoPuerta {
    And,
    Or,
    Not,
    // NUEVO: Resto de puertas lógicas fundamentales
    Nand,
    Nor,
    Xor,
    Xnor,
    // Interruptor de entrada manual
    Interruptor,
    // Elemento receptor final (LED / Bombilla)
    Led,
    // Generador de pulsos de reloj
    Reloj,
    // Display de 7 segmentos con 7 entradas directas (A..G)
    Display7Seg,
    // Display de 7 segmentos con decodificador de 4 bits (8, 4, 2, 1)
    Display4Bits,
}

// Ahora el nodo recuerda qué operación matemática debe hacer
#[derive(Component)]
pub struct NodoLogico {
    pub tipo: TipoPuerta,
}

// Identifica las dos barras laterales donde se pegan las patillas del Chip
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum BarraChip {
    IzquierdaEntradas,
    DerechaSalidas,
}

// Marca los Interruptores o LEDs que están "pegados" a una barra de Chip
#[derive(Component)]
pub struct PegadoABarra(pub BarraChip);

// Identifica el cuadrado izquierdo que alterna entre 0 y 1
#[derive(Component)]
pub struct BotonInterruptor {
    pub estado: bool,
}

// Identifica el cuadrado izquierdo del reloj que controla su frecuencia
#[derive(Component)]
pub struct ControlReloj {
    pub estado: bool,
    pub indice_frec: usize,
    pub acumulador: f32,
}

// Identifica el texto que muestra los Hz dentro del reloj
#[derive(Component)]
pub struct TextoReloj;

// Identifica el bloque luminoso del LED de salida
#[derive(Component)]
pub struct IndicadorLed;

// Identifica el número de patilla de entrada (0, 1, 2...)
#[derive(Component)]
pub struct PinEntrada(pub usize);

// Identifica cada uno de los 7 segmentos luminosos (0 = A ... 6 = G)
#[derive(Component)]
pub struct SegmentoDisplay(pub usize);

// Identifica el interior oscuro del conector para verse "hueco"
#[derive(Component)]
pub struct CentroHueco;

#[derive(Component)]
pub struct Arrastrando;

#[derive(Component)]
pub struct CamaraPrincipal;
// --- Bloque 5: Componentes de Conectores ---
#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum TipoConector {
    Entrada,
    Salida,
}

// Almacena el estado lógico actual (true = 1, false = 0)
#[derive(Component)]
pub struct ValorSeñal(pub bool);

// --- Bloque 6: Componentes de Cables ---
#[derive(Component)]
pub struct Cable {
    pub origen: Entity,
    pub destino: Entity,
    // Lista de codos intermedios por donde pasa el cable
    pub puntos_intermedios: Vec<Vec2>,
}

// --- Bloque 7: Componentes del Menú Contextual ---
#[derive(Component)]
pub struct MenuContextual {
    // Guardamos dónde se hizo clic derecho para crear ahí la pieza
    pub pos_creacion: Vec2,
}

// Identifica la columna secundaria del submenú y recuerda su categoría activa
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
    GuardarCircuito,
    CargarCircuito,
    LimpiarTodo,
}

#[derive(Component)]
pub struct BotonMenu {
    pub accion: AccionMenu,
    // Coordenada mundial directa del centro del botón para detección infalible
    pub centro_mundo: Vec2,
}

// Ventana flotante para teclear el nombre del fichero al guardar
#[derive(Component)]
pub struct DialogoArchivo {
    pub es_guardar: bool,
    pub nombre: String,
}

// Etiqueta de texto dinámico dentro del cuadro de diálogo
#[derive(Component)]
pub struct TextoInputArchivo;

// Ventana contenedora de la lista de ficheros .cir encontrados
#[derive(Component)]
pub struct SelectorCarga;

// Botón individual para cada fichero .cir de la lista
#[derive(Component)]
pub struct BotonArchivoCir {
    pub nombre_archivo: String,
    pub centro_mundo: Vec2,
}