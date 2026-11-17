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

// ==========================================
// Fichero: src/componentes.rs
// Bloque: Bloque 3: Recursos Globales
// ==========================================
#[derive(Resource, Default)]
pub struct EstadoCrearLinea {
    pub origen_entidad: Option<Entity>,
    pub origen_pos: Vec2,
    pub puntos_intermedios: Vec<Vec2>,
    pub es_bus: bool,
    pub num_hilos_bus: usize,
}

#[derive(Resource, Default)]
pub struct CodoArrastrado {
    pub seleccion: Option<(Entity, usize)>,
    pub seleccion_bus: Vec<(Entity, usize)>,
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

// Controla el rectángulo de selección múltiple con arrastre de ratón izquierdo
#[derive(Resource, Default)]
pub struct CajaSeleccion {
    pub activa: bool,
    pub inicio: Vec2,
    pub actual: Vec2,
}

// ==========================================
// Fichero: src/componentes.rs
// Bloque: Bloque 4: Componentes de Nodos, Etiquetas, Buses y Chips
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
    ConectorBus,
    Chip,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ModoDireccionBus {
    #[default]
    HaciaBus,      // "->" Las N patillas son Entradas que vuelcan al bus
    DesdeBus,      // "<-" Las N patillas son Salidas que leen del bus
    Bidireccional, // "<->" Híbrido
}

impl ModoDireccionBus {
    pub fn siguiente(self) -> Self {
        match self {
            Self::HaciaBus => Self::DesdeBus,
            Self::DesdeBus => Self::Bidireccional,
            Self::Bidireccional => Self::HaciaBus,
        }
    }

    pub fn texto_flecha(self) -> &'static str {
        match self {
            Self::HaciaBus => "->",
            Self::DesdeBus => "<-",
            Self::Bidireccional => "<->",
        }
    }

    pub fn a_u8(self) -> u8 {
        match self {
            Self::HaciaBus => 0,
            Self::DesdeBus => 1,
            Self::Bidireccional => 2,
        }
    }

    pub fn desde_u8(v: u8) -> Self {
        match v {
            1 => Self::DesdeBus,
            2 => Self::Bidireccional,
            _ => Self::HaciaBus,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct DatosConectorBus {
    pub num_hilos: usize,
    pub rotacion: u8, // 0 = Derecha (0°), 1 = Arriba (90°), 2 = Izquierda (180°), 3 = Abajo (270°)
    pub modo: ModoDireccionBus,
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
    pub cables_bus: Vec<(usize, usize)>,
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
            } else if let Some(cfg_bus) = n.cod_tipo.strip_prefix("BUS:") {
                let n_hilos = cfg_bus
                    .split(':')
                    .next()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(8);
                salidas_nodos.push(vec![false; n_hilos.max(1)]);
                sub_redes.push(None);
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

// Botón en el cuello del Conector de Bus donde se muestra la flecha (-> / <- / <->)
#[derive(Component)]
pub struct ControlFlechaBus {
    pub nodo_padre: Entity,
}

// Botones [+] y [-] integrados en el Conector de Bus para regular el número de conexiones N
#[derive(Component)]
pub struct BotonReguladorBus {
    pub nodo_padre: Entity,
    pub delta: isize,
}

// Puerto circular en el extremo del cuello donde se conectará el cable grueso del Bus en el Paso 2
#[derive(Component)]
pub struct PuertoTroncoBus {
    pub nodo_padre: Entity,
}

#[derive(Component)]
pub struct Arrastrando {
    pub offset: Vec2,
}

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
// Bloque: Bloque 6: Componentes de Conectores, Cables y Buses
// ==========================================
#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum TipoConector {
    Entrada,
    Salida,
}

// Marca las patillas de un Conector de Bus en modo "<->" para que admitan tanto
// iniciar cables (como Salida) como recibir cables que vienen de otra Salida (como Entrada)
#[derive(Component)]
pub struct ConectorBidireccional;

#[derive(Component, Default)]
pub struct ValorSeñal(pub bool);

#[derive(Component)]
pub struct Cable {
    pub origen: Entity,
    pub destino: Entity,
    pub puntos_intermedios: Vec<Vec2>,
}

// Cable grueso multicolor de N hilos que une dos Conectores de Bus (con soporte de uniones en T)
#[derive(Component, Clone)]
pub struct CableBus {
    pub origen_nodo: Entity,
    pub destino_nodo: Entity,
    pub puntos_intermedios: Vec<Vec2>,
    pub num_hilos: usize,
    pub estados: Vec<bool>,
}

// --- Bloque 7: Componentes del Menú Contextual, Diálogos y Barras UI ---
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
    Bus,
    Chip,
    Archivo,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccionMenu {
    AbrirCategoria(CategoriaMenu),
    CrearNodo(TipoPuerta),
    CrearConectorBus(usize),
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
    pub texto_actual: String,
    pub es_chip: bool,
}

#[derive(Component)]
pub struct TextoInputArchivo;

#[derive(Component)]
pub struct DialogoEtiquetaPin {
    pub nodo_objetivo: Entity,
    pub texto_actual: String,
}

#[derive(Component)]
pub struct TextoInputEtiqueta;

#[derive(Component)]
pub struct SelectorCarga;

#[derive(Component)]
pub struct BotonArchivoCarga {
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