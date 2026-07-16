"""Configure or render the orbital Blender scene as an application-ready MP4.

Configure only (updates the .blend file):
    blender --background .temp/blender/ark_orbital_scene.blend \
      --python tools/blender/render_ark_orbital_loop.py

Render the 12-second loop:
    blender --background .temp/blender/ark_orbital_scene.blend \
      --python tools/blender/render_ark_orbital_loop.py -- --render
"""

from __future__ import annotations

import sys
import shutil
import subprocess
from pathlib import Path

import bpy


ROOT = Path(__file__).resolve().parents[2]
SCENE_PATH = ROOT / ".temp" / "blender" / "ark_orbital_scene.blend"
VIDEO_PATH = ROOT / "frontend" / "public" / "assets" / "ark-orbital-loop.mp4"
FRAME_DIR = ROOT / ".temp" / "blender" / "frames"
FRAME_PREFIX = FRAME_DIR / "ark_orbital_"


def configure_render() -> None:
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x = 1280
    scene.render.resolution_y = 720
    scene.render.resolution_percentage = 100
    scene.render.fps = 30
    scene.frame_start = 1
    scene.frame_end = 360
    # Blender 5.2 renders an image sequence. FFmpeg encodes it afterwards;
    # retaining frames makes the long render recoverable and inspectable.
    scene.render.image_settings.file_format = "PNG"
    scene.render.filepath = str(FRAME_PREFIX)
    bpy.ops.wm.save_as_mainfile(filepath=str(SCENE_PATH))


def main() -> None:
    if not SCENE_PATH.exists():
        raise RuntimeError(f"Scene file not found: {SCENE_PATH}")

    configure_render()
    if "--render" in sys.argv:
        FRAME_DIR.mkdir(parents=True, exist_ok=True)
        bpy.ops.render.render(animation=True)
        ffmpeg = shutil.which("ffmpeg")
        if not ffmpeg:
            raise RuntimeError("FFmpeg is required to encode the rendered PNG sequence.")
        VIDEO_PATH.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            [
                ffmpeg,
                "-y",
                "-framerate",
                "30",
                "-start_number",
                "1",
                "-i",
                str(FRAME_DIR / "ark_orbital_%04d.png"),
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                "-crf",
                "18",
                "-movflags",
                "+faststart",
                str(VIDEO_PATH),
            ],
            check=True,
        )
        print(f"Rendered menu loop: {VIDEO_PATH}")
    else:
        print(f"Configured render output: {VIDEO_PATH}")


if __name__ == "__main__":
    main()
