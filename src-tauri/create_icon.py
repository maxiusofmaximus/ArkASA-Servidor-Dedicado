from PIL import Image
import os

# Create a simple 32x32 white transparent image
img = Image.new('RGBA', (32, 32), (255, 255, 255, 0))

# Save as ICO
os.makedirs('icons', exist_ok=True)
img.save('icons/icon.ico')
print(f"Icon created: {img.size[0]}x{img.size[1]}, {os.path.getsize('icons/icon.ico')} bytes")
