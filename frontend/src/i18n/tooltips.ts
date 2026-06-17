// Tooltip descriptions for all settings and UI controls.
// Key matches labelToKey(label) from translations.ts.
// Descriptions are in Spanish by default (main language).

const tooltips: Record<string, string> = {
  // ── ActionBar buttons ───────────────────────────────────────────────────────
  'back':                     'Volver a la pestaña anterior',
  'restore_defaults':         'Restablecer todos los valores a los predeterminados del servidor ARK',
  'choose_difficulty':        'Elegir un perfil de dificultad predefinido (fácil / normal / difícil / etc.)',
  'save_settings':            'Guardar la configuración actual en el archivo TOML del servidor. En modo autoguardado esto ocurre automáticamente al cambiar cualquier valor.',
  'autosave':                 'El autoguardado está activo — cada cambio se guarda automáticamente tras 1,5 s sin modificaciones. Cambia a guardado manual en Opciones.',
  'import_config':            'Importar un archivo .toml de configuración de ARK. Se mostrará un diff de los cambios antes de aplicar.',
  'options':                  'Abrir opciones: idioma, guardado, proveedor de backup, on-demand servers, etc. (Atajo: Escape)',
  'logs':                     'Abrir / cerrar el visor de logs del servidor en tiempo real. Muestra los archivos de log de los mapas activos.',
  'stop_server':              'Detener el servidor dedicado ARK. Los jugadores conectados serán desconectados.',
  'start_server':             'Iniciar el servidor dedicado ARK con la configuración actual. Si hay cambios sin guardar se guardarán automáticamente.',
  'update':                   'Descargar e instalar actualizaciones para los mods seleccionados',
  'get_mods':                 'Descargar e instalar los mods activos desde Steam/CurseForge',
  'deactivate_mod':           'Quitar el mod seleccionado de la lista de mods activos del servidor',
  'mod_info':                 'Ver información detallada del mod seleccionado',

  // ── Player settings ─────────────────────────────────────────────────────────
  'damage_multiplier':        'Multiplica el daño que infligen los jugadores. 1.0 = valor base. 2.0 = el doble de daño.',
  'resistance_multiplier':    'Multiplica la resistencia al daño de los jugadores. Valores menores a 1.0 aumentan el daño recibido; mayores lo reducen.',
  'water_drain':              'Multiplica la velocidad a la que los jugadores consumen agua. 0.5 = mitad de velocidad; 2.0 = doble.',
  'food_drain':               'Multiplica la velocidad a la que los jugadores consumen comida. 0.5 = hambre más lenta; 2.0 = hambre más rápida.',
  'stamina_drain':            'Multiplica la velocidad de consumo de stamina al correr, saltar, etc. Valores menores = stamina dura más.',
  'health_recovery':          'Multiplica la velocidad de recuperación de salud de los jugadores. 2.0 = el doble de regen.',
  'harvesting_damage':        'Multiplica el daño causado a los recursos al cosecharlos (árboles, rocas, etc.). Afecta la cantidad de recursos obtenidos.',

  // ── Creature settings ───────────────────────────────────────────────────────
  'max_count':                'Número máximo de criaturas salvajes que pueden existir simultáneamente en el servidor. Valores altos requieren más RAM/CPU.',
  'disable_taming':           'Desactiva completamente la domesticación de criaturas. Útil para servidores PVP puros.',
  'disable_riding':           'Desactiva montar criaturas. Los dinos se pueden domesticar pero no se pueden usar como montura.',

  // ── Structure settings ──────────────────────────────────────────────────────
  'disable_placement_collision': 'Permite colocar estructuras aunque se solapen con otras. Útil para builds complejos pero puede causar glitches.',
  'repair_cooldown':          'Segundos de cooldown entre reparaciones de estructuras en zona PVP. 0 = sin límite de reparación.',

  // ── World rules ─────────────────────────────────────────────────────────────
  'xp_multiplier':            'Multiplicador global de puntos de experiencia obtenidos. Afecta matar, cosechar, craftear y explorar.',
  'taming_speed':             'Multiplica la velocidad de domesticación de criaturas. 5.0 = 5× más rápido. No afecta la efectividad del knock-out.',
  'harvest_yield':            'Multiplica la cantidad de recursos obtenidos al cosechar. 3.0 = el triple de madera, piedra, metal, etc.',
  'allow_speed_leveling':     'Permite a los jugadores invertir puntos de nivel en el atributo Velocidad de movimiento.',
  'allow_flyer_speed_leveling':'Permite subir la velocidad de los pterosaurios y otros voladores al ganar niveles. Importante para servidores de voladores rápidos.',
  'maximum_difficulty':       'Nivel máximo de criaturas salvajes. Con dificultad oficial 5.0, el nivel máximo posible es 150.',
  'pve_mode':                 'Activa el modo PVE: los jugadores no pueden atacarse entre sí. Las tribus compiten solo contra el entorno y los jefes.',
  'hardcore_mode':            'En modo hardcore, la muerte del personaje es permanente (sin respawn). El perfil se elimina al morir.',

  // ── Server rules ────────────────────────────────────────────────────────────
  'allow_third_person_camera': 'Permite a los jugadores cambiar a cámara en tercera persona presionando la tecla correspondiente.',
  'global_voice_chat':        'Activa el chat de voz de rango global — todos los jugadores del servidor se escuchan entre sí independientemente de la distancia.',
  'proximity_chat':           'Activa el chat de voz de proximidad — solo los jugadores cercanos se escuchan.',
  'admin_logging':            'Registra en el log del servidor todos los comandos de administrador ejecutados (cheat, etc.).',
  'notify_player_joined':     'Muestra un aviso en pantalla a todos los jugadores cuando alguien se conecta al servidor.',
  'notify_player_left':       'Muestra un aviso en pantalla cuando un jugador abandona el servidor.',
  'server_crosshair':         'Muestra la mira de apuntado en pantalla. Algunos jugadores lo prefieren desactivado para mayor inmersión.',
  'show_floating_damage':     'Muestra los números de daño flotantes sobre las criaturas y jugadores al recibir impactos.',

  // ── PVE settings ────────────────────────────────────────────────────────────
  'allow_cave_building':      'Permite construir estructuras dentro de cuevas en modo PVE. Si está desactivado, las cuevas son solo para exploración.',
  'force_allow_cave_flyers':  'Permite a los voladores entrar en cuevas. Por defecto, la mayoría de criaturas voladoras no pueden entrar en zonas de cueva.',
  'allow_flyer_carry':        'Permite a los pterosaurios y otros voladores llevar a jugadores y criaturas en sus garras.',
  'disable_structure_decay':  'Desactiva el deterioro automático de estructuras por tiempo. Útil para servidores con jugadores ocasionales.',
  'structure_decay_period':   'Multiplica el tiempo antes de que las estructuras desprotegidas comiencen a deteriorarse. 2.0 = el doble de tiempo.',
  'disable_dino_decay':       'Desactiva el deterioro de dinos reclamados por inactividad en modo PVE.',
  'dino_decay_period':        'Multiplica el tiempo antes de que los dinos sin actividad sean marcados para deterioro.',
  'prevent_tribe_alliances':  'Impide que las tribus formen alianzas formales. Útil para servidores muy competitivos.',
  'pve_allow_tribe_war':      'Permite a las tribus en PVE declararse la guerra entre sí para combate temporal.',
  'pve_tribe_war_cancel':     'Permite cancelar una guerra de tribus activa en PVE.',
  'extra_structure_prevention_volumes': 'Activa volúmenes adicionales que previenen la construcción en ciertas áreas del mapa.',
  'prevent_diseases':         'Impide que los jugadores contraigan enfermedades como la gripe o la lepra.',
  'non-permanent_diseases':   'Las enfermedades se curan solas al pasar tiempo o por la curación natural, sin necesidad de antídotos.',

  // ── PVP settings ────────────────────────────────────────────────────────────
  'pvp_dino_decay':           'Los dinos reclamados que no hayan sido accedidos en cierto tiempo pueden ser reclamados por otros jugadores en PVP.',
  'zone_structure_damage':    'Multiplica el daño a estructuras dentro de zonas PVP específicas del mapa.',

  // ── World advanced ──────────────────────────────────────────────────────────
  'day_cycle_speed':          'Multiplica la velocidad del ciclo completo día/noche. Valores altos = días y noches más cortos.',
  'day_time_speed':           'Multiplica la velocidad solo durante el día. Independiente de la noche.',
  'night_time_speed':         'Multiplica la velocidad solo durante la noche. Útil para acortar noches sin afectar el día.',
  'spoiling_time':            'Multiplica el tiempo que tardan los alimentos en descomponerse. 2.0 = duran el doble. Útil para servidores con poco tiempo de juego.',
  'item_decomposition':       'Multiplica el tiempo que tardan los ítems caídos en el suelo en desaparecer.',
  'corpse_decomposition':     'Multiplica el tiempo que tarda un cadáver en desaparecer. Mayor tiempo = más oportunidades de recuperar equipo.',
  'no_resource_radius_(players)':    'Radio alrededor de los jugadores donde los recursos no reaparecen. En metros.',
  'no_resource_radius_(structures)': 'Radio alrededor de las estructuras donde los recursos no reaparecen.',
  'harvest_health':           'Multiplica la cantidad de golpes necesarios para cosechar un recurso (vida del recurso).',
  'resource_respawn':         'Multiplica el tiempo de reaparición de recursos cosechados. 0.5 = reaparecen dos veces más rápido.',
  'crop_growth_speed':        'Multiplica la velocidad de crecimiento de cultivos en maceteros y huertos.',
  'crop_decay_speed':         'Multiplica la velocidad de deterioro de cultivos sin agua o fertilizante.',
  'mating_interval':          'Multiplica el tiempo de espera entre ciclos de cría de una criatura. 0.1 = crías 10× más frecuentes.',
  'lay_egg_interval':         'Multiplica el intervalo entre la puesta de huevos no fertilizados de las criaturas.',
  'egg_hatch_speed':          'Multiplica la velocidad de incubación de huevos. 10.0 = 10× más rápido.',
  'poop_interval':            'Multiplica el intervalo entre deposiciones de las criaturas. Afecta la producción de fertilizante.',
  'baby_mature_speed':        'Multiplica la velocidad de maduración de crías. 20.0 = las crías maduran 20× más rápido.',
  'baby_food_consumption':    'Multiplica la velocidad de consumo de comida de las crías. Útil cuando baby_mature_speed es alto.',
  'baby_cuddle_interval':     'Multiplica el intervalo entre interacciones de imprinting con crías. Menor = más frecuente.',
  'baby_cuddle_grace_period': 'Tiempo de gracia después de un imprinting perdido antes de penalizar el porcentaje de imprinting.',
  'baby_cuddle_loss_(imprint)':'Porcentaje de imprinting perdido si se falla una interacción de cuddling.',
  'baby_imprint_stat_scale':  'Escala el bonus de estadísticas que otorga el imprinting completo. 1.0 = hasta +30% según la especie.',
  'force_reset_wild_dinos':   'Fuerza el respawn de todas las criaturas salvajes al reiniciar el servidor. Elimina dinos de bajo nivel acumulados.',

  // ── Wild/Player stats per level ─────────────────────────────────────────────
  'health_per_level':         'Bonus de salud ganado por cada punto de nivel. Afecta tanto al nivel salvaje como al nivel ganado tras domesticar.',
  'stamina_per_level':        'Bonus de stamina por nivel. Mayor valor = las criaturas aguantan más antes de agotarse.',
  'oxygen_per_level':         'Bonus de oxígeno por nivel. Afecta el tiempo que pueden aguantar bajo el agua.',
  'food_per_level':           'Bonus de capacidad de comida por nivel. Mayor capacidad = se alimentan menos frecuentemente.',
  'water_per_level':          'Bonus de capacidad de agua por nivel.',
  'weight_per_level':         'Bonus de peso máximo de carga por nivel. Afecta cuánto puede cargar el dino.',
  'melee_damage_per_level':   'Bonus de daño cuerpo a cuerpo por nivel. Afecta los ataques físicos de la criatura.',
  'speed_per_level':          'Bonus de velocidad de movimiento por nivel. Solo aplicable si Allow Speed Leveling está activo.',
  'fortitude_per_level':      'Bonus de resistencia a temperaturas extremas por nivel. Reduce el efecto de calor/frío.',
  'torpidity_per_level':      'Bonus de torpor por nivel. El torpor sube al recibir ataques paralizantes.',
  'crafting_speed_per_level': 'Bonus de velocidad de crafteo por nivel. Solo existe para los personajes de jugador.',

  // ── XP multipliers ──────────────────────────────────────────────────────────
  'generic_xp':               'Multiplicador base de XP para acciones que no tienen categoría específica.',
  'kill_xp':                  'Multiplicador de XP obtenido al matar criaturas salvajes.',
  'harvest_xp':               'Multiplicador de XP obtenido al cosechar recursos (madera, piedra, metal, etc.).',
  'craft_xp':                 'Multiplicador de XP obtenido al craftear objetos.',
  'special_xp':               'Multiplicador de XP de acciones especiales como activar beacons o completar desafíos.',
  'explorer_note_xp':         'Multiplicador de XP obtenido al leer notas de explorador (diarios de los survivors).',
  'boss_kill_xp':             'Multiplicador de XP obtenido al derrotar a los jefes del juego (Broodmother, Dragon, etc.).',
  'alpha_kill_xp':            'Multiplicador de XP obtenido al matar criaturas alfa (más grandes y peligrosas).',
  'wild_kill_xp':             'Multiplicador de XP obtenido al matar dinos salvajes cualquiera.',
  'cave_kill_xp':             'Multiplicador de XP obtenido por matar en el interior de cuevas.',
  'tamed_kill_xp':            'Multiplicador de XP obtenido al matar con dinos domesticados.',

  // ── Misc settings ───────────────────────────────────────────────────────────
  'allow_custom_recipes':     'Permite a los jugadores crear recetas personalizadas en el Tomo de Recetas.',
  'custom_recipe_effectiveness': 'Multiplica la efectividad (calidad) de las recetas personalizadas. 2.0 = el doble de efecto.',
  'custom_recipe_skill':      'Multiplica la habilidad de crafteo usada en el cálculo de recetas personalizadas.',
  'supply_crate_loot_quality': 'Multiplica la calidad del botín en cajas de suministro (beacons). 2.0 = equipo de mayor calidad.',
  'fishing_loot_quality':     'Multiplica la calidad del botín obtenido al pescar. Afecta la rareza de los planos obtenidos.',
  'crafting_speed':           'Multiplica la velocidad de crafteo en la inventario y estaciones de trabajo.',
  'max_tamed_dinos':          'Número máximo de dinos domesticados que puede haber en el servidor (de todos los jugadores combinados). Por defecto: 6000.',
  'platform_structure_limit': 'Número máximo de estructuras que se pueden colocar en una silla de montura de plataforma (Brontosaurio, etc.).',
  'disable_friendly_fire':    'Impide que los jugadores de la misma tribu se dañen entre sí (y sus dinos).',
  'no_survivor_downloads':    'Impide la descarga de personajes superviviente desde otro servidor del clúster.',
  'no_dino_downloads':        'Impide la descarga de dinos desde otro servidor del clúster.',
  'no_item_downloads':        'Impide la descarga de ítems desde otro servidor del clúster.',
  'disable_dino_riding':      'Impide montar cualquier criatura domesticada.',
  'disable_dino_taming':      'Desactiva completamente el sistema de domesticación. Las criaturas no pueden ser tameadas.',
  'allow_raid_dino_feeding':  'Permite alimentar dinos asignados para raids (Titanosaurio, etc.) sin requerir permisos de tribu.',
  'disable_photo_mode':       'Desactiva el modo foto del juego (la función de captura en tercera persona ampliada).',
  'photo_mode_range_limit':   'Distancia máxima de la cámara en modo foto desde el personaje. 0 = sin límite.',
  'disable_structure_collision': 'Desactiva la colisión entre estructuras al colocarlas. Permite construcciones superpuestas.',
}

export default tooltips
export type { tooltips as TooltipDict }
