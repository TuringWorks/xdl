#!/usr/bin/env python3

import os
import subprocess
import shutil
import time
import glob

# Directory for generated images
IMAGES_DIR = "examples_images"
os.makedirs(IMAGES_DIR, exist_ok=True)

# Get visual examples from our earlier results
visual_files = []
with open('examples_output/visual_files.txt', 'r') as f:
    for line in f:
        if line.strip():
            visual_files.append(line.strip().replace('VISUAL: ', ''))

print(f"Found {len(visual_files)} visual examples to process")

# Process each visual example
for i, example_file in enumerate(visual_files):
    if not os.path.exists(example_file):
        print(f"File not found: {example_file}")
        continue

    basename = os.path.basename(example_file).replace('.xdl', '').replace('.m', '').replace('.pro', '')
    print(f"[{i+1}/{len(visual_files)}] Processing {basename}...")

    # Clean up any existing xdl_*.png files before running
    for old_file in glob.glob("xdl_*.png"):
        try:
            os.remove(old_file)
        except:
            pass

    try:
        # Run the example
        result = subprocess.run(['xdl', example_file],
                              capture_output=True, text=True, timeout=60)

        if result.returncode == 0:
            print(f"  ✓ Successfully executed {basename}")

            # Look for generated image files and rename them
            generated_images = []
            for png_file in glob.glob("xdl_*.png"):
                new_name = f"{basename}_{os.path.basename(png_file)}"
                dest_path = os.path.join(IMAGES_DIR, new_name)
                shutil.move(png_file, dest_path)
                generated_images.append(new_name)
                print(f"  → Saved image: {new_name}")

            if not generated_images:
                print(f"  ⚠ No images generated for {basename}")
        else:
            print(f"  ✗ Failed to execute {basename}: {result.stderr[:100]}...")

    except subprocess.TimeoutExpired:
        print(f"  ✗ Timeout executing {basename}")
    except Exception as e:
        print(f"  ✗ Error processing {basename}: {e}")

    # Small delay between runs
    time.sleep(1)

# Create an index of generated images
image_index = {}
for root, dirs, files in os.walk(IMAGES_DIR):
    for file in files:
        if file.endswith(('.png', '.svg', '.jpg', '.jpeg')):
            base_name = file.split('_')[0]
            if base_name not in image_index:
                image_index[base_name] = []
            image_index[base_name].append(file)

# Save the image index
with open(os.path.join(IMAGES_DIR, 'image_index.txt'), 'w') as f:
    f.write("XDL Example Images Index\n")
    f.write("=" * 50 + "\n\n")
    for example, images in sorted(image_index.items()):
        f.write(f"{example}:\n")
        for img in images:
            f.write(f"  - {img}\n")
        f.write("\n")

print(f"\nImage generation complete!")
print(f"Images saved to: {IMAGES_DIR}")
print(f"Generated images for {len(image_index)} examples")
print(f"Total images: {sum(len(imgs) for imgs in image_index.values())}")