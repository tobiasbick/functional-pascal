# [Documentation](../doc/captures.md#verification)
"""Check the capture and crop commands using their public file formats."""

import os
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
import unittest

from PIL import Image, ImageChops


APP = Path(__file__).resolve().parents[1]
PROJECT = APP / "GuiDocCaptures.fpasproj"


def run(*arguments):
    return subprocess.run(
        ("dotnet", "run", "--project", PROJECT, "--no-build", "--", *arguments),
        capture_output=True, text=True, timeout=120,
    )


class CropTests(unittest.TestCase):
    def test_crop_preserves_the_selected_pixels(self):
        with TemporaryDirectory() as directory:
            frames = Path(directory)
            output = frames / "output"
            with Image.new("RGBA", (4, 3), "red") as frame:
                frame.putpixel((2, 1), (1, 2, 3, 128))
                frame.putpixel((3, 2), (31, 52, 73, 0))
                frame.save(frames / "widget.png")
            (frames / "widget.crop").write_text("1,1,4,3", encoding="utf-8")

            result = run("--crop", frames, output)
            self.assertEqual(result.returncode, 0, result.stderr)
            with Image.open(output / "widget.png") as image:
                self.assertEqual((image.mode, image.size), ("RGB", (3, 2)))
                self.assertEqual(image.getpixel((0, 0)), (255, 0, 0))
                self.assertEqual(image.getpixel((1, 0)), (1, 2, 3))
                self.assertEqual(image.getpixel((2, 1)), (31, 52, 73))
            with Image.open(frames / "widget.png") as source:
                self.assertEqual((source.mode, source.size), ("RGBA", (4, 3)))

    def test_rejects_missing_out_of_frame_and_blank_regions(self):
        with TemporaryDirectory() as directory:
            frames = Path(directory)
            output = frames / "output"
            with Image.new("RGB", (4, 3), "white") as frame:
                frame.save(frames / "widget.png")

            for bounds, reason in [
                (None, "no crop files"),
                ("0,0,wrong,3", "Invalid crop coordinates"),
                ("0,0,5,3", "crop outside the frame"),
                ("1,0,1,2", "crop outside the frame"),
                ("0,0,4,3", "blank widget capture"),
            ]:
                with self.subTest(bounds=bounds):
                    if bounds is not None:
                        (frames / "widget.crop").write_text(bounds, encoding="utf-8")
                    result = run("--crop", frames, output)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn(reason, result.stderr)
                    self.assertFalse((output / "widget.png").exists())

            result = run("--crop", frames, frames)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("different directory", result.stderr)


class GalleryTests(unittest.TestCase):
    @unittest.skipIf(sys.platform.startswith("linux") and not os.environ.get("DISPLAY"),
                     "The gallery needs an X display on Linux.")
    def test_default_chapter_publishes_frames_and_crop_bounds(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            frames = root / "frames"
            output = root / "widgets"
            result = run(frames)
            self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
            regions = {file.stem for file in frames.glob("*.crop")}
            expected_widgets = {"button", "dropdown-menu", "confirm-button", "swipe-actions"}
            self.assertTrue(expected_widgets <= regions, expected_widgets - regions)
            self.assertEqual(regions, {file.stem for file in frames.glob("*.png")})

            # Actions captures these controls before the next input or frame advance.
            with Image.open(frames / "button.png") as first, Image.open(frames / "hold-button.png") as later:
                self.assertEqual(first.size, later.size)
                self.assertTrue(first.convert("RGBA").tobytes() == later.convert("RGBA").tobytes(),
                                "Consecutive captures of a static frame produced different pixels.")

            result = run("--crop", frames, output)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(regions, {file.stem for file in output.glob("*.png")})
            for name in regions:
                with self.subTest(name=name):
                    coordinates = (frames / f"{name}.crop").read_text(encoding="utf-8")
                    bounds = tuple(map(int, coordinates.split(",")))
                    with Image.open(frames / f"{name}.png") as frame, Image.open(output / f"{name}.png") as image:
                        expected = frame.crop(bounds).convert("RGB")
                        self.assertEqual(image.size, expected.size)
                        self.assertEqual(image.mode, "RGB")
                        self.assertIsNone(ImageChops.difference(image, expected).getbbox())


if __name__ == "__main__":
    unittest.main()
