#!/usr/bin/env python3
"""
Gemini Image Tool — generate and analyze images using Nano Banana Pro.

Uses Nano Banana Pro (Gemini 3 Pro Image) for highest quality image generation
and analysis. All generated images are post-processed with ImageMagick to add
a gentle vignette with edge colors matching the specified background.

Usage:
  # Generate an image
  python3 tools/gemini-image/gemini_image.py generate \
    --prompt "Album art for a folk song in A major" \
    --output cover.png \
    --bg-color "#1a1a2e"

  # Analyze an image
  python3 tools/gemini-image/gemini_image.py analyze \
    --input photo.png \
    --prompt "Describe the musical mood this image evokes"

  # Edit an image (send image + text prompt, get new image back)
  python3 tools/gemini-image/gemini_image.py edit \
    --input source.png \
    --prompt "Make this more rustic and folk-inspired" \
    --output edited.png \
    --bg-color "#1a1a2e"

Environment:
  GEMINI_API_KEY — Google AI API key (from GCP project)
"""

import argparse
import os
import subprocess
import sys
from pathlib import Path

from google import genai
from google.genai import types


# Nano Banana Pro — highest quality image generation, March 2026
IMAGE_MODEL = "gemini-3-pro-image-preview"

# Fallback for analysis-only tasks (cheaper, still excellent)
ANALYSIS_MODEL = "gemini-3.1-pro"


def apply_vignette(image_path: str, bg_color: str = "#000000") -> None:
    """
    Apply a gentle vignette using ImageMagick that fades edges to bg_color.
    This ensures images blend seamlessly into web page backgrounds.
    """
    # Create a vignette mask and composite with background color
    # -vignette 0x80 creates a gentle oval fade
    # The background color is set so edges match the target page color
    try:
        subprocess.run(
            [
                "magick", image_path,
                # Set the background to the target color
                "-background", bg_color,
                # Create a gentle vignette (radius x sigma)
                "-vignette", "0x80",
                # Flatten to apply the background color at edges
                "-flatten",
                image_path,
            ],
            check=True, capture_output=True, text=True, timeout=30,
        )
        print(f"Applied vignette with edge color {bg_color}")
    except FileNotFoundError:
        print("Warning: ImageMagick 'magick' not found. Skipping vignette.",
              file=sys.stderr)
        print("Install with: brew install imagemagick", file=sys.stderr)
    except subprocess.CalledProcessError as e:
        print(f"Warning: vignette failed: {e.stderr}", file=sys.stderr)


def generate_image(prompt: str, output_path: str, bg_color: str,
                   api_key: str) -> None:
    """Generate an image using Nano Banana Pro."""
    client = genai.Client(api_key=api_key)

    print(f"Generating image with {IMAGE_MODEL}...")
    print(f"Prompt: {prompt}")

    response = client.models.generate_content(
        model=IMAGE_MODEL,
        contents=[prompt],
        config=types.GenerateContentConfig(
            response_modalities=["image", "text"],
        ),
    )

    saved = False
    for part in response.parts:
        if part.inline_data is not None:
            image = part.as_image()
            image.save(output_path)
            print(f"Image saved to {output_path}")
            saved = True
            break
        elif part.text is not None:
            print(f"Model text: {part.text}")

    if not saved:
        print("ERROR: No image was generated", file=sys.stderr)
        sys.exit(1)

    # Post-process with vignette
    apply_vignette(output_path, bg_color)
    print(f"Done: {output_path}")


def analyze_image(input_path: str, prompt: str, api_key: str) -> str:
    """Analyze an image using Gemini 3.1 Pro (most powerful model)."""
    client = genai.Client(api_key=api_key)

    print(f"Analyzing image with {ANALYSIS_MODEL}...")

    # Read the image
    from PIL import Image
    img = Image.open(input_path)

    response = client.models.generate_content(
        model=ANALYSIS_MODEL,
        contents=[prompt, img],
    )

    result = response.text
    print(result)
    return result


def edit_image(input_path: str, prompt: str, output_path: str,
               bg_color: str, api_key: str) -> None:
    """Edit an image by sending it with a text prompt to Nano Banana Pro."""
    client = genai.Client(api_key=api_key)

    print(f"Editing image with {IMAGE_MODEL}...")

    from PIL import Image
    img = Image.open(input_path)

    response = client.models.generate_content(
        model=IMAGE_MODEL,
        contents=[prompt, img],
        config=types.GenerateContentConfig(
            response_modalities=["image", "text"],
        ),
    )

    saved = False
    for part in response.parts:
        if part.inline_data is not None:
            result_image = part.as_image()
            result_image.save(output_path)
            print(f"Edited image saved to {output_path}")
            saved = True
            break
        elif part.text is not None:
            print(f"Model text: {part.text}")

    if not saved:
        print("ERROR: No edited image was generated", file=sys.stderr)
        sys.exit(1)

    apply_vignette(output_path, bg_color)
    print(f"Done: {output_path}")


def main():
    parser = argparse.ArgumentParser(
        description="Gemini Image Tool — generate and analyze images"
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    # Generate
    gen_parser = subparsers.add_parser("generate", help="Generate an image")
    gen_parser.add_argument("--prompt", required=True, help="Image prompt")
    gen_parser.add_argument("--output", required=True, help="Output file path")
    gen_parser.add_argument("--bg-color", default="#000000",
                            help="Background color for vignette edges")

    # Analyze
    ana_parser = subparsers.add_parser("analyze", help="Analyze an image")
    ana_parser.add_argument("--input", required=True, help="Input image path")
    ana_parser.add_argument("--prompt", default="Describe this image in detail",
                            help="Analysis prompt")

    # Edit
    edit_parser = subparsers.add_parser("edit", help="Edit an image")
    edit_parser.add_argument("--input", required=True, help="Input image path")
    edit_parser.add_argument("--prompt", required=True, help="Edit instruction")
    edit_parser.add_argument("--output", required=True, help="Output file path")
    edit_parser.add_argument("--bg-color", default="#000000",
                             help="Background color for vignette edges")

    args = parser.parse_args()

    api_key = os.environ.get("GEMINI_API_KEY", "")
    if not api_key:
        # Try reading from a local config file
        key_file = Path.home() / ".config" / "gemini" / "api_key"
        if key_file.exists():
            api_key = key_file.read_text().strip()

    if not api_key:
        print("ERROR: Set GEMINI_API_KEY or create ~/.config/gemini/api_key",
              file=sys.stderr)
        sys.exit(1)

    if args.command == "generate":
        generate_image(args.prompt, args.output, args.bg_color, api_key)
    elif args.command == "analyze":
        analyze_image(args.input, args.prompt, api_key)
    elif args.command == "edit":
        edit_image(args.input, args.prompt, args.output, args.bg_color,
                   api_key)


if __name__ == "__main__":
    main()
