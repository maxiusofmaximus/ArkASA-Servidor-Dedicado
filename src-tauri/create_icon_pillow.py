#!/usr/bin/env python
import sys
import subprocess

# Try to install Pillow
try:
    from PIL import Image
except ImportError:
    print("Installing Pillow...")
    subprocess.check_call([sys.executable, "-m", "pip", "install", "-q", "pillow"])
    from PIL import Image

import os

# Create a 32x32 white transparent image
img = Image.new('RGBA', (32, 32), (255, 255, 255, 0))

# Ensure directory exists
os.makedirs('icons', exist_ok=True)

# Save as ICO
img.save('icons/icon.ico')
size = os.path.getsize('icons/icon.ico')
print(f"Icon created: {size} bytes")

# Verify it's a valid ICO
with open('icons/icon.ico', 'rb') as f:
    header = f.read(6)
    if header[0:2] == b'\x00\x00' and header[2:4] == b'\x01\x00':
        print("Icon format: Valid ICO")
    else:
        print(f"Warning: Unexpected header bytes: {header.hex()}")
