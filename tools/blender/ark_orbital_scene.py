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

FPS = 30
FRAME_END = 720
# The reference keeps the world mostly in shadow: the planet is an occluding
# silhouette, not a brightly lit object at the centre of the artwork.
PLANET_RADIUS = 1.92
PLANET_CENTER_Z = -0.66

CYAN = (0.02, 0.72, 1.0, 1.0)
VIOLET = (0.32, 0.06, 0.92, 1.0)
MAGENTA = (1.0, 0.03, 0.56, 1.0)
WHITE_BLUE = (0.45, 0.9, 1.0, 1.0)


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
    material = bpy.data.materials.new("Planet obsidian")
    material.use_nodes = True
    principled = material.node_tree.nodes.get("Principled BSDF")
    principled.inputs["Base Color"].default_value = (0.001, 0.006, 0.014, 1)
    principled.inputs["Metallic"].default_value = 0.48
    principled.inputs["Roughness"].default_value = 0.7
    return material


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
    spline = curve.splines.new("NURBS")
    spline.points.add(len(points) - 1)
    for point, coordinate in zip(spline.points, points):
        point.co = (*coordinate, 1.0)
    spline.order_u = min(3, len(points))
    spline.use_endpoint_u = not cyclic
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


def add_starfield(material: bpy.types.Material) -> None:
    random.seed(71)
    for index in range(620):
        radius = random.uniform(8.0, 20.0)
        theta = random.uniform(0.0, math.tau)
        phi = random.uniform(0.17, math.pi - 0.17)
        location = Vector(
            (
                radius * math.sin(phi) * math.cos(theta),
                radius * math.sin(phi) * math.sin(theta) + 2.0,
                radius * math.cos(phi),
            )
        )
        bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1, radius=random.uniform(0.004, 0.018), location=location)
        star = bpy.context.object
        star.name = f"star_{index:03d}"
        star.data.materials.append(material)


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
        radial = random.triangular(3.25, 5.05, 4.25)
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
            random.uniform(0.017, 0.048),
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
            if distance < 0.75 and random.random() > 0.26:
                add_curve(
                    f"{name}_link_{index:03d}_{other_index:03d}",
                    [point, other],
                    palette[(index + other_index) % len(palette)],
                    0.0028,
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


def add_energy_cluster(
    name: str,
    center_angle: float,
    travel_angle: float,
    palette: list[bpy.types.Material],
    phase: int,
) -> None:
    """Add a moving swarm, the visual unit that sells the orbital motion."""
    random.seed(480 + phase)
    swarm = bpy.data.objects.new(name, None)
    bpy.context.collection.objects.link(swarm)

    # Dense, uneven crystals emulate the active portions of the reference
    # matrix. Their local depth makes the field feel layered in motion.
    for index in range(72):
        theta = random.uniform(0, math.tau)
        distance = random.triangular(0.012, 0.58, 0.11)
        local_position = Vector(
            (
                math.cos(theta) * distance,
                random.uniform(-0.16, 0.08),
                math.sin(theta) * distance * random.uniform(0.55, 1.15),
            )
        )
        if index % 3:
            bpy.ops.mesh.primitive_ico_sphere_add(
                subdivisions=1,
                radius=random.uniform(0.006, 0.032),
                location=local_position,
            )
        else:
            bpy.ops.mesh.primitive_cube_add(size=1, location=local_position)
        shard = bpy.context.object
        shard.name = f"{name}_shard_{index:02d}"
        shard.scale = (random.uniform(0.008, 0.048), random.uniform(0.004, 0.016), random.uniform(0.008, 0.07))
        shard.rotation_euler = (random.random() * math.tau, random.random() * math.tau, random.random() * math.tau)
        shard.data.materials.append(palette[index % len(palette)])
        shard.parent = swarm

    # Bright connectors make the swarm read as an electrically active matrix.
    for index in range(10):
        angle = index * math.tau / 5 + random.uniform(-0.15, 0.15)
        length = random.uniform(0.07, 0.34)
        add_curve(
            f"{name}_spark_{index:02d}",
            [Vector((0, 0, 0)), Vector((math.cos(angle) * length, 0, math.sin(angle) * length * 0.74))],
            palette[index % len(palette)],
            0.004,
            swarm,
        )

    # The start/end pose is identical so the rendered loop closes cleanly.
    keyframes = (
        (1, center_angle),
        (121, center_angle + travel_angle),
        (241, center_angle - travel_angle * 0.72),
        (360, center_angle),
    )
    for frame, angle in keyframes:
        swarm.location = ring_point(angle, 4.27, 3.63, 0.32)
        swarm.scale = (1.0, 1.0, 1.0) if frame in (1, 360) else (1.18, 1.18, 1.18)
        swarm.keyframe_insert("location", frame=frame)
        swarm.keyframe_insert("scale", frame=frame)


def point_camera(camera: bpy.types.Object, target: Vector) -> None:
    direction = target - camera.location
    camera.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()


def configure_scene() -> None:
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x = 1920
    scene.render.resolution_y = 1080
    scene.render.resolution_percentage = 50
    scene.render.image_settings.file_format = "PNG"
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
    violet = emission_material("Violet energy", VIOLET, 2.25)
    magenta = emission_material("Magenta energy", MAGENTA, 2.4)
    white_blue = emission_material("Cold rim", WHITE_BLUE, 3.25)
    muted_cyan = muted_emission_material("Distant cyan matrix", CYAN, 0.95)
    muted_violet = muted_emission_material("Distant violet matrix", VIOLET, 0.82)
    muted_magenta = muted_emission_material("Distant magenta matrix", MAGENTA, 0.82)
    star = emission_material("Stars", (0.28, 0.5, 0.9, 1), 1.75)

    add_starfield(star)

    bpy.ops.mesh.primitive_uv_sphere_add(
        segments=128,
        ring_count=64,
        radius=PLANET_RADIUS,
        location=(0, 0, PLANET_CENTER_Z),
    )
    planet = bpy.context.object
    planet.name = "Obsidian planet"
    planet.data.materials.append(planet_material())

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

    # Four imperfect clouds leave large empty areas. This reads as a layered
    # planetary network rather than a thin ring around a logo.
    add_matrix_cloud("upper left matrix", math.radians(142), math.radians(48), 88, [muted_violet, muted_magenta, muted_cyan], 1)
    add_matrix_cloud("upper right matrix", math.radians(43), math.radians(38), 68, [muted_cyan, muted_violet, muted_magenta], 2)
    add_matrix_cloud("lower left matrix", math.radians(232), math.radians(44), 76, [muted_magenta, muted_violet, muted_cyan], 3)
    add_matrix_cloud("lower right matrix", math.radians(317), math.radians(34), 58, [muted_violet, muted_magenta, muted_cyan], 4)
    add_energy_cluster("upper left energy field", math.radians(133), math.radians(22), [magenta, violet, cyan], 1)
    add_energy_cluster("right energy field", math.radians(-2), math.radians(25), [cyan, violet, magenta], 2)
    add_energy_cluster("lower left energy field", math.radians(237), math.radians(20), [violet, magenta, cyan], 3)
    add_energy_cluster("lower right energy field", math.radians(294), math.radians(15), [magenta, violet, cyan], 4)

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
