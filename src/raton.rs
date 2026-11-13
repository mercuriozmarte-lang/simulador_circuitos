// ==========================================
// Fichero: src/raton.rs
// ==========================================

// --- Bloque 1: Importaciones ---
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::componentes::*;
use crate::configuracion::ConfigApp;
use crate::archivo::{PeticionGuardar, PeticionCargar};
use crate::{
    crear_puerta, 
    crear_interruptor, 
    crear_led,
    crear_reloj,
    crear_display,
};

// --- Bloque 2A: Gestión de Hover y Clics en el Menú Contextual ---
pub fn gestionar_clic_menu(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    q_nodos: Query<Entity, With<NodoLogico>>,
    q_cables: Query<Entity, With<Cable>>,
    q_menu: Query<(Entity, &Transform, &MenuContextual)>,
    q_submenu: Query<(Entity, &SubMenuContextual)>,
    mut q_botones_menu: Query<(&BotonMenu, &mut Sprite)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    // Si no hay menú abierto, salimos
    let Ok((ent_menu, trf_menu, menu)) = q_menu.single() else {
        return;
    };

    let Ok(window) = q_windows.single() else { return; };
    let Ok((cam, cam_trf)) = q_camera.single() else { return; };

    let Some(pos_pantalla) = window.cursor_position() else {
        return;
    };
    let Ok(pos_mundo) = cam.viewport_to_world_2d(
        cam_trf, 
        pos_pantalla
    ) else {
        return;
    };

    // Comprobamos sobre qué botón del menú o submenú está el ratón ahora mismo
    // usando su coordenada mundial exacta (sin depender de GlobalTransform)
    let mut accion_bajo_cursor = None;
    let mitad_ancho = 65.0;
    let mitad_alto = 16.0;

    for (btn_menu, mut sprite) in q_botones_menu.iter_mut() {
        let pos_btn = btn_menu.centro_mundo;
        let dentro_x = (pos_mundo.x - pos_btn.x).abs() < mitad_ancho;
        let dentro_y = (pos_mundo.y - pos_btn.y).abs() < mitad_alto;

        if dentro_x && dentro_y {
            accion_bajo_cursor = Some(btn_menu.accion);
            // Iluminamos el botón bajo el cursor para dar feedback visual claro
            sprite.color = Color::srgb(0.38, 0.48, 0.62);
        } else {
            sprite.color = Color::srgb(0.24, 0.28, 0.36);
        }
    }

    // 1. HOVER AUTOMÁTICO: Si el cursor está sobre una categoría principal,
    // abrimos su submenú a la derecha inmediatamente
    if let Some(AccionMenu::AbrirCategoria(cat)) = accion_bajo_cursor {
        let mut ya_abierto = false;
        for (ent_sub, sub) in q_submenu.iter() {
            if sub.categoria == cat {
                ya_abierto = true;
            } else {
                commands.entity(ent_sub).despawn();
            }
        }

        if !ya_abierto {
            let pos_base = trf_menu.translation.truncate();
            abrir_submenu(&mut commands, pos_base, cat);
        }
    }

    // 2. CLIC IZQUIERDO: Ejecutamos la acción del submenú o cerramos si clica fuera
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    if let Some(accion) = accion_bajo_cursor {
        match accion {
            AccionMenu::AbrirCategoria(cat) => {
                // Si también hace clic en la categoría, abrimos su submenú
                // y mantenemos el menú abierto
                for (ent_sub, sub) in q_submenu.iter() {
                    if sub.categoria != cat {
                        commands.entity(ent_sub).despawn();
                    }
                }
                let pos_base = trf_menu.translation.truncate();
                abrir_submenu(&mut commands, pos_base, cat);
                return;
            }
            AccionMenu::CrearNodo(tipo) => {
                let cuadricula = 50.0;
                let pos_snap = Vec2::new(
                    (menu.pos_creacion.x / cuadricula).round() 
                        * cuadricula,
                    (menu.pos_creacion.y / cuadricula).round() 
                        * cuadricula,
                );

                match tipo {
                    TipoPuerta::And 
                    | TipoPuerta::Or 
                    | TipoPuerta::Not => {
                        crear_puerta(
                            &mut commands, 
                            pos_snap, 
                            tipo, 
                            &config
                        );
                    }
                    TipoPuerta::Interruptor => {
                        crear_interruptor(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            pos_snap,
                            false,
                            &config,
                        );
                    }
                    TipoPuerta::Led => {
                        crear_led(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            pos_snap,
                            &config,
                        );
                    }
                    TipoPuerta::Reloj => {
                        crear_reloj(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            pos_snap,
                            1,
                            &config,
                        );
                    }
                    TipoPuerta::Display4Bits => {
                        crear_display(
                            &mut commands,
                            pos_snap,
                            true,
                            &config,
                        );
                    }
                    TipoPuerta::Display7Seg => {
                        crear_display(
                            &mut commands,
                            pos_snap,
                            false,
                            &config,
                        );
                    }
                }
            }
            AccionMenu::GuardarCircuito => {
                commands.spawn(PeticionGuardar);
            }
            AccionMenu::CargarCircuito => {
                commands.spawn(PeticionCargar);
            }
            AccionMenu::LimpiarTodo => {
                for ent_n in q_nodos.iter() {
                    commands.entity(ent_n).despawn();
                }
                for ent_c in q_cables.iter() {
                    commands.entity(ent_c).despawn();
                }
            }
        }
    }

    // Cerramos menú principal y submenú tras ejecutar una acción final o clicar fuera
    commands.entity(ent_menu).despawn();
    for (ent_sub, _) in q_submenu.iter() {
        commands.entity(ent_sub).despawn();
    }
}

// --- Bloque 2B: Clic en Nodos y Conectores ---
pub fn detectar_clic_nodo(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    // Si el menú está abierto, ignoramos clics sobre el lienzo
    q_menu: Query<Entity, With<MenuContextual>>,
    q_nodos: Query<(Entity, &Transform), With<NodoLogico>>,
    q_conectores: Query<
        (Entity, &GlobalTransform, &TipoConector)
    >,
    // Consulta para el cuadrado interactivo del interruptor
    mut q_botones: Query<
        (&GlobalTransform, &mut BotonInterruptor, &mut Sprite),
        Without<ControlReloj>,
    >,
    // Consulta para el cuadrado interactivo de frecuencia del reloj
    mut q_relojes: Query<
        (&GlobalTransform, &mut ControlReloj, &mut Sprite, &Children),
        Without<BotonInterruptor>,
    >,
    mut q_textos_reloj: Query<&mut Text2d, With<TextoReloj>>,
    mut estado_linea: ResMut<EstadoCrearLinea>,
    mut next_state: ResMut<NextState<ModoApp>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    // Si había un menú contextual en pantalla, no interactuamos con las puertas de debajo
    if !q_menu.is_empty() {
        return;
    }

    let Ok(window) = q_windows.single() else { return; };
    let Ok((cam, cam_trf)) = q_camera.single() else { return; };

    if let Some(pos_pantalla) = window.cursor_position() {
        if let Ok(pos_mundo) = cam.viewport_to_world_2d(
            cam_trf, 
            pos_pantalla
        ) {
            // 1. Prioridad 1: Clic en conectores para tirar cables
            for (ent_conector, g_trf, tipo) in q_conectores.iter() {
                let pos_conector = g_trf.translation().truncate();
                
                let dist = pos_mundo.distance(pos_conector);
                if dist < config.rad_clic_con {
                    if *tipo == TipoConector::Salida {
                        estado_linea.origen_entidad = Some(ent_conector);
                        estado_linea.origen_pos = pos_conector;
                        next_state.set(ModoApp::CreandoLinea);
                        return; 
                    }
                }
            }

            // 2. Prioridad 2: Clic en el cuadrado de cambio de estado (0 / 1)
            for (g_trf, mut boton, mut sprite) in q_botones.iter_mut() {
                let pos_boton = g_trf.translation().truncate();
                if (pos_mundo.x - pos_boton.x).abs() < 13.0 
                    && (pos_mundo.y - pos_boton.y).abs() < 13.0 
                {
                    boton.estado = !boton.estado;
                    if boton.estado {
                        sprite.color = config.col_estado_1;
                    } else {
                        sprite.color = config.col_estado_0;
                    }
                    return;
                }
            }

            // 2B. Prioridad 2B: Clic en el cuadrado de frecuencia del Reloj
            let etiquetas_frec = ["OFF", "1Hz", "2Hz", "5Hz", "10Hz"];
            for (g_trf, mut reloj, mut sprite, hijos) in q_relojes.iter_mut() {
                let pos_ctrl = g_trf.translation().truncate();
                if (pos_mundo.x - pos_ctrl.x).abs() < 17.0 
                    && (pos_mundo.y - pos_ctrl.y).abs() < 13.0 
                {
                    reloj.indice_frec = 
                        (reloj.indice_frec + 1) % etiquetas_frec.len();
                    reloj.acumulador = 0.0;

                    if reloj.indice_frec == 0 {
                        reloj.estado = false;
                        sprite.color = config.col_estado_0;
                    }

                    for hijo in hijos.iter() {
                        if let Ok(mut txt) = q_textos_reloj.get_mut(hijo) {
                            txt.0 = 
                                etiquetas_frec[reloj.indice_frec].to_string();
                        }
                    }
                    return;
                }
            }

            // 3. Prioridad 3: Clic en el resto del elemento para moverlo
            for (entidad, trf) in q_nodos.iter() {
                let centro_nodo = trf.translation.truncate();
                if pos_mundo.distance(centro_nodo) < config.rad_clic_nodo {
                    commands.entity(entidad).insert(Arrastrando);
                    return; 
                }
            }
        }
    }
}

// --- Bloque 3: Soltar Ratón ---
pub fn soltar_nodo(
    buttons: Res<ButtonInput<MouseButton>>,
    mut q_arrastrando: Query<
        (Entity, &mut Transform), 
        With<Arrastrando>
    >,
    q_conectores: Query<
        (Entity, &GlobalTransform, &TipoConector)
    >,
    q_cables: Query<&Cable>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    estado_app: Res<State<ModoApp>>,
    estado_linea: Res<EstadoCrearLinea>,
    mut next_state: ResMut<NextState<ModoApp>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if !buttons.just_released(MouseButton::Left) {
        return;
    }

    // 1. Soltar nodos y aplicar el encaje magnético (Snap to Grid)
    for (entidad, mut trf) in q_arrastrando.iter_mut() {
        let cuadricula = 50.0;
        
        trf.translation.x = 
            (trf.translation.x / cuadricula).round() * cuadricula;
        trf.translation.y = 
            (trf.translation.y / cuadricula).round() * cuadricula;
        
        commands.entity(entidad).remove::<Arrastrando>();
    }

    // 2. Lógica de conexión de cables
    if *estado_app.get() == ModoApp::CreandoLinea {
        let Ok(window) = q_windows.single() else { 
            next_state.set(ModoApp::Edicion);
            return; 
        };
        let Ok((cam, cam_trf)) = q_camera.single() else { 
            next_state.set(ModoApp::Edicion);
            return; 
        };

        if let Some(pos_pantalla) = window.cursor_position() {
            if let Ok(pos_mundo) = cam.viewport_to_world_2d(
                cam_trf, 
                pos_pantalla
            ) {
                for (ent_destino, g_trf, tipo) in q_conectores.iter() {
                    let pos_conector = g_trf.translation().truncate();
                    
                    let dist = pos_mundo.distance(pos_conector);
                    if dist < config.rad_clic_con {
                        if *tipo == TipoConector::Entrada {
                            let entrada_ocupada = q_cables
                                .iter()
                                .any(|c| c.destino == ent_destino);

                            if !entrada_ocupada {
                                if let Some(orig) = 
                                    estado_linea.origen_entidad 
                                {
                                    commands.spawn(Cable {
                                        origen: orig,
                                        destino: ent_destino,
                                    });
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
        
        next_state.set(ModoApp::Edicion);
    }
}

// --- Bloque 4: Mover Nodo Arrastrado ---
pub fn mover_nodo_arrastrado(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    mut q_arrastrando: Query<&mut Transform, With<Arrastrando>>,
) {
    let Ok(window) = q_windows.single() else { return; };
    let Ok((cam, cam_trf)) = q_camera.single() else { return; };

    if let Some(pos_pantalla) = window.cursor_position() {
        if let Ok(pos_mundo) = cam.viewport_to_world_2d(
            cam_trf, 
            pos_pantalla
        ) {
            for mut trf in q_arrastrando.iter_mut() {
                trf.translation.x = pos_mundo.x;
                trf.translation.y = pos_mundo.y;
            }
        }
    }
}

// --- Bloque 5: Borrar o Abrir Menú con Clic Derecho ---
pub fn borrar_con_clic_derecho(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<
        (&Camera, &GlobalTransform), 
        With<CamaraPrincipal>
    >,
    q_nodos: Query<(Entity, &Transform), With<NodoLogico>>,
    q_cables: Query<(Entity, &Cable)>,
    q_global_trf: Query<&GlobalTransform>,
    q_menu: Query<Entity, With<MenuContextual>>,
    q_submenu: Query<Entity, With<SubMenuContextual>>,
    mut commands: Commands,
    config: Res<ConfigApp>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }

    // Si ya había un menú o submenú abierto, los cerramos primero
    for ent_menu in q_menu.iter() {
        commands.entity(ent_menu).despawn();
    }
    for ent_sub in q_submenu.iter() {
        commands.entity(ent_sub).despawn();
    }

    let Ok(window) = q_windows.single() else { return; };
    let Ok((cam, cam_trf)) = q_camera.single() else { return; };

    if let Some(pos_pantalla) = window.cursor_position() {
        if let Ok(pos_mundo) = cam.viewport_to_world_2d(
            cam_trf, 
            pos_pantalla
        ) {
            // 1. Intentamos borrar una puerta lógica si el ratón está encima
            for (ent_nodo, trf) in q_nodos.iter() {
                let centro = trf.translation.truncate();
                if pos_mundo.distance(centro) < config.rad_clic_nodo {
                    commands.entity(ent_nodo).despawn();
                    return;
                }
            }

            // 2. Si no tocamos puerta, comprobamos si tocamos algún cable
            for (ent_cable, cable) in q_cables.iter() {
                if let (Ok(trf_ori), Ok(trf_des)) = (
                    q_global_trf.get(cable.origen),
                    q_global_trf.get(cable.destino),
                ) {
                    let a = trf_ori.translation().truncate();
                    let b = trf_des.translation().truncate();

                    let ab = b - a;
                    let ap = pos_mundo - a;
                    let t = (ap.dot(ab) / ab.length_squared())
                        .clamp(0.0, 1.0);
                    let punto_mas_cercano = a + ab * t;

                    if pos_mundo.distance(punto_mas_cercano) < 8.0 {
                        commands.entity(ent_cable).despawn();
                        return;
                    }
                }
            }

            // 3. Si estamos en vacío, abrimos el menú principal compacto
            abrir_menu_contextual(&mut commands, pos_mundo);
        }
    }
}

// --- Bloque 6: Creador del Menú Principal y Submenús ---
fn abrir_menu_contextual(
    commands: &mut Commands, 
    pos: Vec2
) {
    let categorias = [
        ("Puertas  >>", AccionMenu::AbrirCategoria(CategoriaMenu::Puertas)),
        ("Entradas >>", AccionMenu::AbrirCategoria(CategoriaMenu::Entradas)),
        ("Salidas  >>", AccionMenu::AbrirCategoria(CategoriaMenu::Salidas)),
        ("Archivo  >>", AccionMenu::AbrirCategoria(CategoriaMenu::Archivo)),
    ];

    let ancho_btn = 130.0;
    let alto_btn = 32.0;
    let separacion = 36.0;
    let total_alto = (categorias.len() as f32) * separacion + 10.0;

    commands.spawn((
        Sprite {
            color: Color::srgb(0.12, 0.12, 0.15),
            custom_size: Some(Vec2::new(ancho_btn + 12.0, total_alto)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 5.0),
        MenuContextual { pos_creacion: pos },
    ))
    .with_children(|padre| {
        let inicio_y = 
            ((categorias.len() as f32 - 1.0) * separacion) / 2.0;

        for (i, (texto, accion)) in categorias.iter().enumerate() {
            let y = inicio_y - (i as f32 * separacion);
            let centro_mundo = Vec2::new(pos.x, pos.y + y);

            padre.spawn((
                Sprite {
                    color: Color::srgb(0.24, 0.28, 0.36),
                    custom_size: Some(Vec2::new(ancho_btn, alto_btn)),
                    ..default()
                },
                Transform::from_xyz(0.0, y, 0.1),
                BotonMenu { 
                    accion: *accion,
                    centro_mundo,
                },
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text2d::new(*texto),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(15.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 0.0, 0.1),
                ));
            });
        }
    });
}

fn abrir_submenu(
    commands: &mut Commands,
    pos_menu_principal: Vec2,
    categoria: CategoriaMenu,
) {
    let opciones: Vec<(&str, AccionMenu)> = match categoria {
        CategoriaMenu::Puertas => vec![
            ("Puerta AND", AccionMenu::CrearNodo(TipoPuerta::And)),
            ("Puerta OR", AccionMenu::CrearNodo(TipoPuerta::Or)),
            ("Puerta NOT", AccionMenu::CrearNodo(TipoPuerta::Not)),
        ],
        CategoriaMenu::Entradas => vec![
            ("Interruptor", AccionMenu::CrearNodo(TipoPuerta::Interruptor)),
            ("Reloj (Clock)", AccionMenu::CrearNodo(TipoPuerta::Reloj)),
        ],
        CategoriaMenu::Salidas => vec![
            ("LED Salida", AccionMenu::CrearNodo(TipoPuerta::Led)),
            ("Display 4-Bits", AccionMenu::CrearNodo(TipoPuerta::Display4Bits)),
            ("Display 7-Seg", AccionMenu::CrearNodo(TipoPuerta::Display7Seg)),
        ],
        CategoriaMenu::Archivo => vec![
            ("Guardar", AccionMenu::GuardarCircuito),
            ("Cargar", AccionMenu::CargarCircuito),
            ("Limpiar Todo", AccionMenu::LimpiarTodo),
        ],
    };

    let ancho_btn = 130.0;
    let alto_btn = 32.0;
    let separacion = 36.0;
    let total_alto = (opciones.len() as f32) * separacion + 10.0;

    // Colocamos el submenú justo a la derecha del menú principal
    let pos_sub = Vec2::new(
        pos_menu_principal.x + ancho_btn + 14.0,
        pos_menu_principal.y,
    );

    commands.spawn((
        Sprite {
            color: Color::srgb(0.15, 0.16, 0.20),
            custom_size: Some(Vec2::new(ancho_btn + 12.0, total_alto)),
            ..default()
        },
        Transform::from_xyz(pos_sub.x, pos_sub.y, 5.5),
        SubMenuContextual { categoria },
    ))
    .with_children(|padre| {
        let inicio_y = 
            ((opciones.len() as f32 - 1.0) * separacion) / 2.0;

        for (i, (texto, accion)) in opciones.iter().enumerate() {
            let y = inicio_y - (i as f32 * separacion);
            let centro_mundo = Vec2::new(pos_sub.x, pos_sub.y + y);

            padre.spawn((
                Sprite {
                    color: Color::srgb(0.24, 0.28, 0.36),
                    custom_size: Some(Vec2::new(ancho_btn, alto_btn)),
                    ..default()
                },
                Transform::from_xyz(0.0, y, 0.1),
                BotonMenu { 
                    accion: *accion,
                    centro_mundo,
                },
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text2d::new(*texto),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(15.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 0.0, 0.1),
                ));
            });
        }
    });
}