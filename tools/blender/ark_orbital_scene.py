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
PLANET_RADIUS = 2.25
PLANET_CENTER_Z = -0.78

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
    for index in range(480):
        radius = random.uniform(8.0, 18.0)
        theta = random.uniform(0.0, math.tau)
        phi = random.uniform(0.17, math.pi - 0.17)
        location = Vector(
            (
                radius * math.sin(phi) * math.cos(theta),
                radius * math.sin(phi) * math.sin(theta) + 2.0,
                radius * math.cos(phi),
            )
        )
        bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1, radius=random.uniform(0.006, 0.021), location=location)
        star = bpy.context.object
        star.name = f"star_{index:03d}"
        star.data.materials.append(material)


def ring_point(angle: float, radius_x: float = 4.62, radius_z: float = 3.95, depth: float = 0.85) -> Vector:
    return Vector((math.cos(angle) * radius_x, depth, math.sin(angle) * radius_z))


def add_sector(
    name: str,
    start: float,
    end: float,
    palette: list[bpy.types.Material],
    stream_material: bpy.types.Material,
    phase: int,
) -> None:
    random.seed(100 + phase)
    parent = bpy.data.objects.new(name, None)
    bpy.context.collection.objects.link(parent)

    # Sparse outer filaments frame the denser, irregular node band below.
    for filament in range(2):
        points = []
        for step in range(9):
            t = step / 8
            angle = start + (end - start) * t + random.uniform(-0.038, 0.038)
            radius_x = 4.62 + random.uniform(-0.2, 0.2)
            radius_z = 3.95 + random.uniform(-0.18, 0.18)
            points.append(ring_point(angle, radius_x, radius_z, 0.72 + filament * 0.055))
        add_curve(f"{name}_filament_{filament}", points, palette[filament % len(palette)], 0.009, parent)

    # Local mesh of nodes and short connecting links.
    outer_nodes: list[Vector] = []
    inner_nodes: list[Vector] = []
    for index in range(25):
        t = index / 24
        angle = start + (end - start) * t + random.uniform(-0.065, 0.065)
        outer_node = ring_point(
            angle,
            4.58 + random.uniform(-0.3, 0.3),
            3.93 + random.uniform(-0.28, 0.28),
            0.6 + random.uniform(-0.13, 0.13),
        )
        inner_node = ring_point(
            angle + random.uniform(-0.042, 0.042),
            4.08 + random.uniform(-0.22, 0.22),
            3.48 + random.uniform(-0.2, 0.2),
            0.64 + random.uniform(-0.12, 0.12),
        )
        outer_nodes.append(outer_node)
        inner_nodes.append(inner_node)
        add_hex_node(f"{name}_outer_node_{index:02d}", outer_node, palette[index % len(palette)], random.uniform(0.033, 0.07), parent)
        add_hex_node(f"{name}_inner_node_{index:02d}", inner_node, palette[(index + 1) % len(palette)], random.uniform(0.028, 0.06), parent)

    for index in range(len(outer_nodes) - 1):
        if index % 2 == 0:
            add_curve(f"{name}_outer_link_{index:02d}", [outer_nodes[index], outer_nodes[index + 1]], stream_material, 0.005, parent)
        if index % 3 != 1:
            add_curve(f"{name}_inner_link_{index:02d}", [inner_nodes[index], inner_nodes[index + 1]], palette[index % len(palette)], 0.004, parent)
        if index % 2 == 1:
            add_curve(f"{name}_radial_link_{index:02d}", [outer_nodes[index], inner_nodes[index]], palette[(index + 2) % len(palette)], 0.004, parent)

    # A travelling emissive node follows the sector arc.
    path_points = [ring_point(start + (end - start) * step / 20, 4.72, 4.02, 0.48) for step in range(21)]
    path = add_curve(f"{name}_traveller_path", path_points, stream_material, 0.0, cyclic=False)
    path.data.path_duration = FRAME_END
    path.hide_render = True
    traveller = add_hex_node(f"{name}_traveller", path_points[0], palette[-1], 0.105)
    follow = traveller.constraints.new("FOLLOW_PATH")
    follow.target = path
    follow.use_curve_follow = False
    follow.forward_axis = "FORWARD_X"
    follow.offset_factor = 0.0
    follow.keyframe_insert("offset_factor", frame=1)
    follow.offset_factor = 1.0
    follow.keyframe_insert("offset_factor", frame=FRAME_END)

    # The entire cloud drifts independently, giving orbital parallax.
    parent.rotation_euler = (0, 0, -0.025)
    parent.location = (0, 0, 0)
    parent.keyframe_insert("rotation_euler", frame=1)
    parent.location = (0.22 if phase % 2 else -0.22, 0, -0.14)
    parent.rotation_euler = (0, 0, 0.035)
    parent.keyframe_insert("location", frame=FRAME_END)
    parent.keyframe_insert("rotation_euler", frame=FRAME_END)


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

    # A dense irregular cluster is closer to a moving energy field than a
    # single icon. White cores punctuate the cyan, violet and magenta shards.
    for index in range(20):
        theta = random.uniform(0, math.tau)
        distance = random.triangular(0.018, 0.28, 0.065)
        local_position = Vector(
            (
                math.cos(theta) * distance,
                random.uniform(-0.08, 0.08),
                math.sin(theta) * distance * 0.74,
            )
        )
        bpy.ops.mesh.primitive_ico_sphere_add(
            subdivisions=1,
            radius=random.uniform(0.011, 0.038),
            location=local_position,
        )
        shard = bpy.context.object
        shard.name = f"{name}_shard_{index:02d}"
        shard.data.materials.append(palette[index % len(palette)])
        shard.parent = swarm

    # Bright connectors make the swarm read as an electrically active matrix.
    for index in range(5):
        angle = index * math.tau / 5 + random.uniform(-0.15, 0.15)
        length = random.uniform(0.09, 0.24)
        add_curve(
            f"{name}_spark_{index:02d}",
            [Vector((0, 0, 0)), Vector((math.cos(angle) * length, 0, math.sin(angle) * length * 0.74))],
            palette[index % len(palette)],
            0.006,
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
        swarm.scale = (1.0, 1.0, 1.0) if frame in (1, 360) else (1.22, 1.22, 1.22)
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

    cyan = emission_material("Cyan energy", CYAN, 2.8)
    violet = emission_material("Violet energy", VIOLET, 2.5)
    magenta = emission_material("Magenta energy", MAGENTA, 2.7)
    white_blue = emission_material("Cold rim", WHITE_BLUE, 3.6)
    star = emission_material("Stars", (0.3, 0.55, 0.95, 1), 2.0)

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

    # Thin emissive arcs create the crisp, bright lateral rim from the reference.
    for name, start, end in (("left_rim", math.radians(122), math.radians(238)), ("right_rim", math.radians(-58), math.radians(58))):
        points = [
            Vector((PLANET_RADIUS * math.cos(angle), -0.08, PLANET_RADIUS * math.sin(angle) + PLANET_CENTER_Z))
            for angle in [start + (end - start) * i / 32 for i in range(33)]
        ]
        add_curve(name, points, white_blue, 0.018)

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

    add_sector("left_orbit", math.radians(112), math.radians(250), [violet, magenta, cyan], violet, 1)
    add_sector("right_orbit", math.radians(-65), math.radians(70), [cyan, violet, magenta], cyan, 2)
    add_sector("lower_orbit", math.radians(220), math.radians(326), [magenta, violet, cyan], magenta, 3)
    add_energy_cluster("left energy swarm", math.radians(172), math.radians(29), [magenta, violet, cyan], 1)
    add_energy_cluster("right energy swarm", math.radians(10), math.radians(32), [cyan, violet, magenta], 2)
    add_energy_cluster("lower energy swarm", math.radians(275), math.radians(24), [violet, magenta, cyan], 3)

    # Camera looks down the negative Y axis: X/Z map directly to the UI background.
    bpy.ops.object.camera_add(location=(0, -17.5, 0.15))
    camera = bpy.context.object
    camera.data.lens = 52
    camera.data.sensor_width = 36
    point_camera(camera, Vector((0, 0.6, -0.25)))
    bpy.context.scene.camera = camera

    # Rim lights only: the planet stays matte and predominantly black.
    for name, location, color, energy in (
        ("Left rim light", (-5.4, 2.6, 1.0), (0.25, 0.65, 1.0), 760),
        ("Right rim light", (5.4, 2.8, 0.7), (0.3, 0.8, 1.0), 820),
        ("Top haze", (0, 2.2, 5.8), (0.15, 0.32, 0.75), 240),
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
