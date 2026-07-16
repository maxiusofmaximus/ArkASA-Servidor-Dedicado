"""Build an original orbital sci-fi menu background in Blender.

Run from the repository root once Blender is installed:

    "C:\\Program Files\\Blender Foundation\\Blender 4.5\\blender.exe" \
      --background --python tools/blender/ark_orbital_scene.py

The script saves a reusable .blend file in .temp/blender/. It deliberately
uses original procedural geometry rather than extracting game assets.
"""

from __future__ import annotations

import math
import random
from pathlib import Path

import bpy
from mathutils import Vector


ROOT = Path(__file__).resolve().parents[2]
OUTPUT_DIR = ROOT / ".temp" / "blender"
BLEND_PATH = OUTPUT_DIR / "ark_orbital_scene.blend"
SPACE_TEXTURE_PATH = ROOT / "frontend" / "public" / "assets" / "ark-space-atmosphere-v1.png"

FPS = 30
# The reference's planet takes tens of seconds to turn perceptibly. A 48 s
# loop lets it make one calm full revolution without a visible jump.
LOOP_END = 1440
LOOP_SCALE = LOOP_END / 360
FRAME_END = LOOP_END
# The reference keeps the world mostly in shadow: the planet is an occluding
# silhouette, not a brightly lit object at the centre of the artwork.
PLANET_RADIUS = 1.92
PLANET_CENTER_Z = -0.66

CYAN = (0.02, 0.72, 1.0, 1.0)
VIOLET = (0.32, 0.06, 0.92, 1.0)
MAGENTA = (1.0, 0.03, 0.56, 1.0)
WHITE_BLUE = (0.45, 0.9, 1.0, 1.0)
LIME = (0.28, 1.0, 0.19, 1.0)
AMBER = (1.0, 0.48, 0.06, 1.0)


def loop_frame(reference_frame: int) -> int:
    """Scale the original 12-second timing marks into the longer loop."""
    return round(reference_frame * LOOP_SCALE)


def clear_scene() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    for datablock_collection in (bpy.data.materials, bpy.data.curves, bpy.data.meshes):
        for datablock in datablock_collection:
            if datablock.users == 0:
                datablock_collection.remove(datablock)


def emission_material(name: str, color: tuple[float, float, float, float], strength: float) -> bpy.types.Material:
    material = bpy.data.materials.new(name)
    material.use_nodes = True
    nodes = material.node_tree.nodes
    nodes.clear()
    output = nodes.new("ShaderNodeOutputMaterial")
    emission = nodes.new("ShaderNodeEmission")
    emission.inputs["Color"].default_value = color
    emission.inputs["Strength"].default_value = strength
    material.node_tree.links.new(emission.outputs["Emission"], output.inputs["Surface"])
    return material


def muted_emission_material(
    name: str, color: tuple[float, float, float, float], strength: float
) -> bpy.types.Material:
    """Emission for the distant, deliberately understated matrix."""
    return emission_material(name, (color[0] * 0.52, color[1] * 0.52, color[2] * 0.62, 1), strength)


def planet_material() -> bpy.types.Material:
    """A shadowed, layered planet with relief visible below the rim lights."""
    material = bpy.data.materials.new("Planet obsidian")
    material.use_nodes = True
    nodes = material.node_tree.nodes
    links = material.node_tree.links
    principled = nodes.get("Principled BSDF")
    continents = nodes.new("ShaderNodeTexNoise")
    continents.inputs["Scale"].default_value = 0.92
    continents.inputs["Detail"].default_value = 5.5
    continents.inputs["Roughness"].default_value = 0.7
    continents.inputs["Distortion"].default_value = 0.35
    continental_ramp = nodes.new("ShaderNodeValToRGB")
    continental_ramp.color_ramp.elements[0].position = 0.32
    continental_ramp.color_ramp.elements[0].color = (0.00008, 0.00022, 0.0005, 1)
    continental_ramp.color_ramp.elements[1].position = 0.68
    continental_ramp.color_ramp.elements[1].color = (0.024, 0.053, 0.085, 1)

    relief = nodes.new("ShaderNodeTexNoise")
    relief.inputs["Scale"].default_value = 6.4
    relief.inputs["Detail"].default_value = 8.0
    relief.inputs["Roughness"].default_value = 0.74
    relief.inputs["Distortion"].default_value = 0.18
    relief_ramp = nodes.new("ShaderNodeValToRGB")
    relief_ramp.color_ramp.elements[0].position = 0.38
    relief_ramp.color_ramp.elements[0].color = (0.18, 0.22, 0.3, 1)
    relief_ramp.color_ramp.elements[1].position = 0.69
    relief_ramp.color_ramp.elements[1].color = (0.88, 0.95, 1.0, 1)
    surface_mix = nodes.new("ShaderNodeMixRGB")
    surface_mix.blend_type = "MULTIPLY"
    surface_mix.inputs["Fac"].default_value = 0.62
    surface_mix.inputs[1].default_value = (0.014, 0.029, 0.055, 1)
    bump = nodes.new("ShaderNodeBump")
    bump.inputs["Strength"].default_value = 0.42
    bump.inputs["Distance"].default_value = 0.1
    links.new(continents.outputs["Fac"], continental_ramp.inputs["Fac"])
    links.new(relief.outputs["Fac"], relief_ramp.inputs["Fac"])
    links.new(continental_ramp.outputs["Color"], surface_mix.inputs[1])
    links.new(relief_ramp.outputs["Color"], surface_mix.inputs[2])
    links.new(surface_mix.outputs["Color"], principled.inputs["Base Color"])
    links.new(relief.outputs["Fac"], bump.inputs["Height"])
    links.new(bump.outputs["Normal"], principled.inputs["Normal"])
    principled.inputs["Metallic"].default_value = 0.22
    principled.inputs["Roughness"].default_value = 0.7
    return material


def cloud_veil_material() -> bpy.types.Material:
    """Dim broken cloud bands that only appear over the planet's lit edge."""
    material = bpy.data.materials.new("Planet cloud veil")
    material.use_nodes = True
    material.surface_render_method = "DITHERED"
    nodes = material.node_tree.nodes
    links = material.node_tree.links
    nodes.clear()
    output = nodes.new("ShaderNodeOutputMaterial")
    mix = nodes.new("ShaderNodeMixShader")
    transparent = nodes.new("ShaderNodeBsdfTransparent")
    cloud = nodes.new("ShaderNodeEmission")
    cloud.inputs["Color"].default_value = (0.16, 0.34, 0.54, 1)
    cloud.inputs["Strength"].default_value = 0.18
    noise = nodes.new("ShaderNodeTexNoise")
    noise.inputs["Scale"].default_value = 3.9
    noise.inputs["Detail"].default_value = 6.0
    noise.inputs["Roughness"].default_value = 0.7
    noise.inputs["Distortion"].default_value = 0.45
    mask = nodes.new("ShaderNodeValToRGB")
    mask.color_ramp.elements[0].position = 0.55
    mask.color_ramp.elements[1].position = 0.71
    links.new(noise.outputs["Fac"], mask.inputs["Fac"])
    links.new(mask.outputs["Color"], mix.inputs[0])
    links.new(transparent.outputs[0], mix.inputs[1])
    links.new(cloud.outputs[0], mix.inputs[2])
    links.new(mix.outputs[0], output.inputs["Surface"])
    return material


def nebula_material(
    name: str,
    color: tuple[float, float, float, float],
    strength: float,
) -> bpy.types.Material:
    """A sparse emissive dust cloud for the distant, non-rotating space."""
    material = bpy.data.materials.new(name)
    material.use_nodes = True
    material.surface_render_method = "DITHERED"
    nodes = material.node_tree.nodes
    links = material.node_tree.links
    nodes.clear()
    output = nodes.new("ShaderNodeOutputMaterial")
    mix = nodes.new("ShaderNodeMixShader")
    transparent = nodes.new("ShaderNodeBsdfTransparent")
    emission = nodes.new("ShaderNodeEmission")
    emission.inputs["Color"].default_value = color
    emission.inputs["Strength"].default_value = strength
    texcoords = nodes.new("ShaderNodeTexCoord")
    mapping = nodes.new("ShaderNodeMapping")
    clouds = nodes.new("ShaderNodeTexNoise")
    clouds.inputs["Scale"].default_value = 1.15
    clouds.inputs["Detail"].default_value = 5.0
    clouds.inputs["Roughness"].default_value = 0.72
    wisps = nodes.new("ShaderNodeTexNoise")
    wisps.inputs["Scale"].default_value = 5.4
    wisps.inputs["Detail"].default_value = 3.0
    blend = nodes.new("ShaderNodeMixRGB")
    blend.blend_type = "MULTIPLY"
    blend.inputs["Fac"].default_value = 0.78
    mask = nodes.new("ShaderNodeValToRGB")
    # Two noise layers multiply to values around 0.2–0.4. Keeping the
    # threshold in that range creates wisps, not a rectangular solid card.
    mask.color_ramp.elements[0].position = 0.16
    mask.color_ramp.elements[1].position = 0.46
    links.new(texcoords.outputs["Generated"], mapping.inputs["Vector"])
    links.new(mapping.outputs["Vector"], clouds.inputs["Vector"])
    links.new(mapping.outputs["Vector"], wisps.inputs["Vector"])
    links.new(clouds.outputs["Fac"], blend.inputs[1])
    links.new(wisps.outputs["Fac"], blend.inputs[2])
    links.new(blend.outputs["Color"], mask.inputs["Fac"])
    links.new(mask.outputs["Color"], mix.inputs[0])
    links.new(transparent.outputs[0], mix.inputs[1])
    links.new(emission.outputs[0], mix.inputs[2])
    links.new(mix.outputs[0], output.inputs["Surface"])
    return material


def space_background_material() -> bpy.types.Material:
    """Use the original generated deep-space plate as a full-frame backdrop."""
    if not SPACE_TEXTURE_PATH.exists():
        raise FileNotFoundError(f"Missing generated space texture: {SPACE_TEXTURE_PATH}")
    material = bpy.data.materials.new("Generated deep space backdrop")
    material.use_nodes = True
    nodes = material.node_tree.nodes
    links = material.node_tree.links
    nodes.clear()
    output = nodes.new("ShaderNodeOutputMaterial")
    emission = nodes.new("ShaderNodeEmission")
    emission.inputs["Strength"].default_value = 0.72
    texture = nodes.new("ShaderNodeTexImage")
    texture.image = bpy.data.images.load(str(SPACE_TEXTURE_PATH), check_existing=True)
    links.new(texture.outputs["Color"], emission.inputs["Color"])
    links.new(emission.outputs["Emission"], output.inputs["Surface"])
    return material


def add_space_backdrop() -> None:
    """Place the 16:9 space plate beyond every 3D foreground layer."""
    bpy.ops.mesh.primitive_plane_add(size=2.0, location=(0.0, 13.0, 0.15), rotation=(math.pi / 2, 0, 0))
    backdrop = bpy.context.object
    backdrop.name = "Generated deep-space backdrop"
    backdrop.scale = (12.7, 7.2, 1.0)
    backdrop.data.materials.append(space_background_material())


def add_curve(
    name: str,
    points: list[Vector],
    material: bpy.types.Material,
    bevel: float,
    parent: bpy.types.Object | None = None,
    cyclic: bool = False,
) -> bpy.types.Object:
    curve = bpy.data.curves.new(name, "CURVE")
    curve.dimensions = "3D"
    curve.resolution_u = 2
    curve.bevel_depth = bevel
    curve.bevel_resolution = 2
    # Closed lattice cells need exact planar sides. Motion paths retain NURBS
    # interpolation, while a cyclic cell is rendered as a true polygon.
    spline = curve.splines.new("POLY" if cyclic else "NURBS")
    spline.points.add(len(points) - 1)
    for point, coordinate in zip(spline.points, points):
        point.co = (*coordinate, 1.0)
    if not cyclic:
        spline.order_u = min(3, len(points))
        spline.use_endpoint_u = True
    spline.use_cyclic_u = cyclic
    curve.materials.append(material)
    obj = bpy.data.objects.new(name, curve)
    bpy.context.collection.objects.link(obj)
    obj.parent = parent
    return obj


def add_hex_node(
    name: str,
    location: Vector,
    material: bpy.types.Material,
    scale: float,
    parent: bpy.types.Object | None = None,
) -> bpy.types.Object:
    bpy.ops.mesh.primitive_torus_add(
        major_radius=scale,
        minor_radius=scale * 0.13,
        major_segments=6,
        minor_segments=4,
        location=location,
        rotation=(math.pi / 2, 0, math.pi / 6),
    )
    node = bpy.context.object
    node.name = name
    node.data.materials.append(material)
    node.parent = parent
    return node


def add_distant_points(
    name: str,
    material: bpy.types.Material,
    positions: list[tuple[float, float, float, float]],
) -> None:
    """Render thousands of tiny camera-facing stars as one lightweight mesh."""
    vertices: list[tuple[float, float, float]] = []
    faces: list[tuple[int, int, int, int]] = []
    for x, y, z, size in positions:
        start = len(vertices)
        vertices.extend(
            (
                (x - size, y, z - size),
                (x + size, y, z - size),
                (x + size, y, z + size),
                (x - size, y, z + size),
            )
        )
        faces.append((start, start + 1, start + 2, start + 3))
    mesh = bpy.data.meshes.new(f"{name} mesh")
    mesh.from_pydata(vertices, [], faces)
    mesh.materials.append(material)
    dust = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(dust)


def add_starfield(material: bpy.types.Material) -> None:
    random.seed(71)
    positions = [
        (
            random.uniform(-10.5, 10.5),
            random.uniform(8.0, 15.0),
            random.uniform(-6.8, 7.2),
            random.uniform(0.0025, 0.0105),
        )
        for _ in range(1550)
    ]
    add_distant_points("Deep fixed starfield", material, positions)


def add_colored_stardust(materials: list[bpy.types.Material]) -> None:
    """Build irregular violet/cyan dust patches without opaque cards."""
    cluster_specs = (
        ((-4.7, 1.2), 1.8, 1.95, 135, 163),
        ((4.9, -0.4), 1.35, 2.25, 120, 271),
        ((-2.4, -4.5), 1.8, 0.9, 90, 419),
        ((1.8, 4.55), 2.6, 0.6, 80, 597),
    )
    for cluster_index, ((center_x, center_z), width, height, count, seed) in enumerate(cluster_specs):
        random.seed(seed)
        positions = []
        for _ in range(count):
            positions.append(
                (
                    random.gauss(center_x, width),
                    random.uniform(9.4, 14.5),
                    random.gauss(center_z, height),
                    random.uniform(0.003, 0.012),
                )
            )
        add_distant_points(f"Colored star dust {cluster_index + 1}", materials[cluster_index % len(materials)], positions)


def ring_point(angle: float, radius_x: float = 4.62, radius_z: float = 3.95, depth: float = 0.85) -> Vector:
    return Vector((math.cos(angle) * radius_x, depth, math.sin(angle) * radius_z))


def add_matrix_cloud(
    name: str,
    center_angle: float,
    angular_width: float,
    count: int,
    palette: list[bpy.types.Material],
    phase: int,
) -> None:
    """Create a broken 3D node field, never a readable circular wireframe."""
    random.seed(100 + phase)
    parent = bpy.data.objects.new(name, None)
    bpy.context.collection.objects.link(parent)
    points: list[Vector] = []

    for index in range(count):
        # Nodes occupy a thick, imperfect shell. The depth variation makes
        # parts of the matrix disappear behind the planet and each other.
        angle = center_angle + random.triangular(-angular_width, angular_width, 0.0)
        radial = random.triangular(3.72, 4.92, 4.34)
        point = ring_point(
            angle,
            radial,
            radial * random.uniform(0.76, 0.92),
            random.uniform(0.55, 1.85),
        )
        points.append(point)
        add_hex_node(
            f"{name}_node_{index:03d}",
            point,
            palette[index % len(palette)],
            random.uniform(0.012, 0.036),
            parent,
        )

    # Each node links only to one or two close neighbours. That makes local
    # cells and gaps instead of a decorative, uninterrupted orbit.
    for index, point in enumerate(points):
        neighbours = sorted(
            (
                ((point - other).length, other_index, other)
                for other_index, other in enumerate(points)
                if other_index > index
            ),
            key=lambda item: item[0],
        )
        for distance, other_index, other in neighbours[:2]:
            if distance < 0.64 and random.random() > 0.2:
                add_curve(
                    f"{name}_link_{index:03d}_{other_index:03d}",
                    [point, other],
                    palette[(index + other_index) % len(palette)],
                    0.0015,
                    parent,
                )

    parent.location = (0, 0, 0)
    parent.rotation_euler = (0, 0, -0.026)
    parent.keyframe_insert("location", frame=1)
    parent.keyframe_insert("rotation_euler", frame=1)
    parent.location = (0.17 if phase % 2 else -0.15, 0.0, -0.09)
    parent.rotation_euler = (0, 0, 0.028)
    parent.keyframe_insert("location", frame=360)
    parent.keyframe_insert("rotation_euler", frame=360)


def add_dual_geodesic_lattice(
    radius: float,
    palette: list[bpy.types.Material],
    muted_material: bpy.types.Material,
) -> bpy.types.Object:
    """Create the pentagon/hexagon dual of an icosphere as a single shell.

    An icosphere at subdivision three has 162 vertices. Its dual therefore
    has 12 pentagons and 150 hexagons: a dense closed-cell topology that
    matches the static reference without becoming a random triangular graph.
    """
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=3, radius=1.0)
    source = bpy.context.object
    mesh = source.data
    parent = bpy.data.objects.new("Dual geodesic matrix", None)
    bpy.context.collection.objects.link(parent)
    parent.location = (0.0, 2.4, -0.42)

    face_centers: list[Vector] = []
    faces_by_vertex: list[list[int]] = [[] for _ in mesh.vertices]
    for face_index, face in enumerate(mesh.polygons):
        center = sum((mesh.vertices[index].co for index in face.vertices), Vector()) / len(face.vertices)
        face_centers.append(center.normalized())
        for vertex_index in face.vertices:
            faces_by_vertex[vertex_index].append(face_index)

    random.seed(902)
    active_angles = (math.radians(136), math.radians(3), math.radians(230), math.radians(303))
    for vertex_index, vertex in enumerate(mesh.vertices):
        normal = vertex.co.normalized()
        tangent_seed = Vector((0, 0, 1)) if abs(normal.z) < 0.88 else Vector((1, 0, 0))
        tangent = normal.cross(tangent_seed).normalized()
        bitangent = normal.cross(tangent).normalized()

        ordered_centers = sorted(
            (face_centers[index] for index in faces_by_vertex[vertex_index]),
            key=lambda point: math.atan2(point.dot(bitangent), point.dot(tangent)),
        )
        # Project polygon corners slightly outward to avoid z-fighting and
        # preserve a clean, illuminated honeycomb outline.
        points = [point * radius for point in ordered_centers]
        angle = math.atan2(normal.z, normal.x) % math.tau
        distance_to_active = min(abs((angle - sector + math.pi) % math.tau - math.pi) for sector in active_angles)
        active = distance_to_active < math.radians(25) and random.random() > 0.34
        material = palette[vertex_index % len(palette)] if active else muted_material
        bevel = 0.008 if active else 0.0032
        cell = add_curve(
            f"dual_cell_{vertex_index:03d}_{'pentagon' if len(points) == 5 else 'hexagon'}",
            points,
            material,
            bevel,
            parent,
            cyclic=True,
        )
        # A subtle phase shift prevents all active cells from pulsing together.
        if active:
            cell.scale = (0.985, 0.985, 0.985)
            cell.keyframe_insert("scale", frame=1)
            cell.scale = (1.025, 1.025, 1.025)
            cell.keyframe_insert("scale", frame=loop_frame(121 + (vertex_index % 36)))
            cell.scale = (0.99, 0.99, 0.99)
            cell.keyframe_insert("scale", frame=loop_frame(241 + (vertex_index % 28)))
            cell.scale = (0.985, 0.985, 0.985)
            cell.keyframe_insert("scale", frame=LOOP_END)

    bpy.data.objects.remove(source, do_unlink=True)
    # ARK's matrix is a stable holographic shell. Its perceived motion comes
    # from lighting and active nodes, not from a conspicuous globe rotation.
    parent.rotation_euler = (0.0, 0.0, 0.0)
    return parent


def add_shell_energy_sector(
    name: str,
    center_angle: float,
    palette: list[bpy.types.Material],
    phase: int,
    matrix: bpy.types.Object,
) -> None:
    """Grow a local field of illuminated veins directly on the dual shell."""
    random.seed(480 + phase)
    sector = bpy.data.objects.new(name, None)
    bpy.context.collection.objects.link(sector)
    sector.parent = matrix

    # The field lives on the front hemisphere of the exact same sphere as the
    # pentagon/hexagon lattice. It is intentionally made of local indicators
    # rather than free lines: the reference energises cells and their nodes.
    def shell_point(x: float, z: float) -> Vector:
        shell_radius = 4.37
        y = -math.sqrt(max(0.2, shell_radius * shell_radius - x * x - z * z))
        return Vector((x, y, z))

    center_x = math.cos(center_angle) * 3.05
    center_z = math.sin(center_angle) * 2.7
    for index in range(22):
        x = center_x + random.triangular(-1.0, 1.0, 0.0)
        z = center_z + random.triangular(-0.78, 0.78, 0.0)
        material = palette[(index + phase) % len(palette)]
        if index % 3:
            spark = add_hex_node(
                f"{name}_active_node_{index:02d}",
                shell_point(x, z),
                material,
                random.uniform(0.018, 0.052),
                sector,
            )
        else:
            bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1, radius=random.uniform(0.008, 0.021), location=shell_point(x, z))
            spark = bpy.context.object
            spark.name = f"{name}_spark_{index:02d}"
            spark.data.materials.append(material)
            spark.parent = sector
        if index % 2 == 0:
            base_scale = spark.scale.copy()
            for frame, multiplier in (
                (1, 0.8),
                (loop_frame(132 + index % 30), 1.75),
                (loop_frame(246 + index % 24), 0.55),
                (LOOP_END, 0.8),
            ):
                spark.scale = base_scale * multiplier
                spark.keyframe_insert("scale", frame=frame)


def point_camera(camera: bpy.types.Object, target: Vector) -> None:
    direction = target - camera.location
    camera.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()


def set_linear_keyframes(obj: bpy.types.Object, data_path: str) -> None:
    """Keep cyclic rotations at a calm, constant speed instead of easing."""
    if not obj.animation_data or not obj.animation_data.action:
        return
    action = obj.animation_data.action
    # Blender 5.2 stores curves in layered action channel bags rather than in
    # Action.fcurves (the pre-5.0 API). Iterate all bags to stay portable.
    for layer in action.layers:
        for strip in layer.strips:
            for channelbag in strip.channelbags:
                for fcurve in channelbag.fcurves:
                    if fcurve.data_path == data_path:
                        for keyframe in fcurve.keyframe_points:
                            keyframe.interpolation = "LINEAR"


def configure_scene() -> None:
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x = 1920
    scene.render.resolution_y = 1080
    scene.render.resolution_percentage = 50
    scene.render.image_settings.file_format = "PNG"
    # The background must be baked into RGB. Leaving alpha enabled made the
    # post-process glow contaminate otherwise black space during H.264 export.
    scene.render.film_transparent = False
    scene.render.filepath = str(OUTPUT_DIR / "ark_orbital_preview.png")
    scene.render.fps = FPS
    scene.frame_start = 1
    scene.frame_end = FRAME_END
    world_background = scene.world.node_tree.nodes.get("Background")
    if world_background:
        world_background.inputs["Color"].default_value = (0.0001, 0.0002, 0.0006, 1)
        world_background.inputs["Strength"].default_value = 0.025


def build_scene() -> None:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    clear_scene()
    configure_scene()

    cyan = emission_material("Cyan energy", CYAN, 2.6)
    violet = emission_material("Violet energy", VIOLET, 2.05)
    magenta = emission_material("Magenta energy", MAGENTA, 2.25)
    white_blue = emission_material("Cold rim", WHITE_BLUE, 3.25)
    lime = emission_material("Lime charge", LIME, 3.35)
    amber = emission_material("Amber charge", AMBER, 3.05)
    muted_cyan = muted_emission_material("Distant cyan matrix", CYAN, 0.95)
    muted_violet = muted_emission_material("Distant violet matrix", VIOLET, 0.82)
    muted_magenta = muted_emission_material("Distant magenta matrix", MAGENTA, 0.82)
    star = emission_material("Stars", (0.28, 0.5, 0.9, 1), 1.75)

    add_space_backdrop()

    bpy.ops.mesh.primitive_uv_sphere_add(
        segments=128,
        ring_count=64,
        radius=PLANET_RADIUS,
        location=(0, 0, PLANET_CENTER_Z),
    )
    planet = bpy.context.object
    planet.name = "Obsidian planet"
    planet.data.materials.append(planet_material())
    # Rotate around the vertical screen axis, like a globe turning toward the
    # right. This is intentionally independent from the stationary matrix.
    # A full cycle preserves a perfectly seamless 12-second video loop.
    planet.rotation_euler = (0.0, 0.0, math.radians(9))
    planet.keyframe_insert("rotation_euler", frame=1)
    planet.rotation_euler = (0.0, 0.0, math.radians(369))
    planet.keyframe_insert("rotation_euler", frame=LOOP_END)
    set_linear_keyframes(planet, "rotation_euler")

    # A separate veil moves with the world but has a small phase offset. It
    # supplies slow atmospheric detail without making the shadowed globe read
    # as a bright generic blue sphere.
    bpy.ops.mesh.primitive_uv_sphere_add(
        segments=128,
        ring_count=64,
        radius=PLANET_RADIUS * 1.008,
        location=(0, -0.01, PLANET_CENTER_Z),
    )
    cloud_veil = bpy.context.object
    cloud_veil.name = "Rotating planetary cloud veil"
    cloud_veil.data.materials.append(cloud_veil_material())
    cloud_veil.rotation_euler = (0.0, 0.0, math.radians(27))
    cloud_veil.keyframe_insert("rotation_euler", frame=1)
    cloud_veil.rotation_euler = (0.0, 0.0, math.radians(387))
    cloud_veil.keyframe_insert("rotation_euler", frame=LOOP_END)
    set_linear_keyframes(cloud_veil, "rotation_euler")

    # Broken lateral crescents rather than a complete luminous outline.
    for name, start, end in (
        ("left_rim", math.radians(144), math.radians(212)),
        ("right_rim", math.radians(-34), math.radians(35)),
    ):
        points = [
            Vector((PLANET_RADIUS * math.cos(angle), -0.08, PLANET_RADIUS * math.sin(angle) + PLANET_CENTER_Z))
            for angle in [start + (end - start) * i / 32 for i in range(33)]
        ]
        add_curve(name, points, white_blue, 0.014)

    # A faint atmospheric shell gives the planet its upper cyan halo.
    bpy.ops.mesh.primitive_uv_sphere_add(
        segments=96,
        ring_count=48,
        radius=PLANET_RADIUS * 1.025,
        location=(0, 0.04, PLANET_CENTER_Z),
    )
    atmosphere = bpy.context.object
    atmosphere.name = "Planet atmosphere"
    atmosphere.data.materials.append(emission_material("Atmosphere", (0.06, 0.36, 0.7, 1), 0.22))
    atmosphere.display_type = "WIRE"
    atmosphere.hide_render = True

    matrix = add_dual_geodesic_lattice(4.35, [magenta, violet, cyan], muted_violet)
    # ARK's active fields use a cyan/white core with brief lime and amber
    # overloads, while the surrounding lattice remains predominantly violet.
    add_shell_energy_sector("upper left energy field", math.radians(133), [white_blue, cyan, lime, magenta], 1, matrix)
    add_shell_energy_sector("right energy field", math.radians(-2), [cyan, white_blue, amber, lime, magenta], 2, matrix)
    add_shell_energy_sector("lower left energy field", math.radians(237), [violet, magenta, cyan, amber], 3, matrix)
    add_shell_energy_sector("lower right energy field", math.radians(294), [magenta, violet, cyan, lime], 4, matrix)

    # Camera looks down the negative Y axis: X/Z map directly to the UI background.
    bpy.ops.object.camera_add(location=(0, -17.5, 0.15))
    camera = bpy.context.object
    camera.data.lens = 55
    camera.data.sensor_width = 36
    point_camera(camera, Vector((0, 0.6, -0.25)))
    bpy.context.scene.camera = camera

    # Rim lights only: the planet stays matte and predominantly black.
    for name, location, color, energy in (
        ("Left rim light", (-5.4, 2.6, 1.0), (0.24, 0.62, 1.0), 540),
        ("Right rim light", (5.4, 2.8, 0.7), (0.3, 0.76, 1.0), 590),
        ("Top haze", (0, 2.2, 5.8), (0.15, 0.32, 0.75), 120),
    ):
        bpy.ops.object.light_add(type="AREA", location=location)
        light = bpy.context.object
        light.name = name
        light.data.energy = energy
        light.data.shape = "DISK"
        light.data.size = 4.0
        light.data.color = color
        point_camera(light, Vector((0, 0, PLANET_CENTER_Z)))

    bpy.context.scene.frame_set(1)
    bpy.ops.wm.save_as_mainfile(filepath=str(BLEND_PATH))
    print(f"Saved Blender scene: {BLEND_PATH}")


if __name__ == "__main__":
    build_scene()
