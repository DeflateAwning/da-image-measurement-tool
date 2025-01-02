"""Ruler GUI."""

import argparse
from dataclasses import dataclass

import cv2
import numpy as np


@dataclass(kw_only=True)
class AppGlobalSingleton:
    """Singleton class to store global variables."""

    points: list[tuple[int, int]]
    scale_factor: float | None = None
    img: np.ndarray | None = None
    original_img: np.ndarray | None = None

    # Stores lines and their measurements
    measurements: list[tuple[tuple[int, int], tuple[int, int], float]]


app_globals = AppGlobalSingleton(points=[], measurements=[])


def handle_house_event(
    event: int,
    x: int,
    y: int,
    _flags: int,
    _param: object | None,
) -> None:
    """Handle the mouse events."""
    if app_globals.img is None or app_globals.original_img is None:
        return

    # On left mouse button click, store the point
    if event == cv2.EVENT_LBUTTONDOWN:
        app_globals.points.append((x, y))
        cv2.circle(app_globals.img, (x, y), 5, (0, 0, 255), -1)

        if len(app_globals.points) == 2:  # noqa: PLR2004
            # Draw a line between two points
            cv2.line(
                app_globals.img,
                app_globals.points[0],
                app_globals.points[1],
                (255, 0, 0),
                2,
            )

            if app_globals.scale_factor is None:
                try:
                    # First line sets the scale factor.
                    real_distance_str = input(
                        "Enter the real-world distance (in mm) of the line you just "
                        "drew:",
                    )
                    real_distance = float(real_distance_str)
                    pixel_distance: float = float(
                        np.linalg.norm(
                            np.array(app_globals.points[0])
                            - np.array(app_globals.points[1]),
                        ),
                    )
                    scale_factor = real_distance / pixel_distance
                    print(f"Scale factor set to {scale_factor:.4f} mm per pixel")
                except ValueError:
                    print("Invalid input. Please enter a numeric value.")
                    app_globals.points = []  # Reset points on invalid input
                except:
                    raise
            else:
                # Measure another line using the scale factor
                pixel_distance = float(
                    np.linalg.norm(
                        np.array(app_globals.points[0])
                        - np.array(app_globals.points[1]),
                    ),
                )
                real_distance = pixel_distance * app_globals.scale_factor
                app_globals.measurements.append(
                    (app_globals.points[0], app_globals.points[1], real_distance),
                )
                print(f"Measured distance: {real_distance:.2f} mm")

            # Clear points after two clicks
            app_globals.points = []
            redraw_image()

    elif event == cv2.EVENT_RBUTTONDOWN:
        # Remove the most recent measurement on right-click
        if app_globals.measurements:
            app_globals.measurements.pop()
            print("Last measurement removed.")
            redraw_image()


def redraw_image() -> None:
    """Redraw the image with all measurements."""
    if app_globals.original_img is None:
        return

    # Reset to the original image
    app_globals.img = app_globals.original_img.copy()

    # Redraw all measurements
    for start, end, distance in app_globals.measurements:
        cv2.line(app_globals.img, start, end, (255, 0, 0), 2)
        mid_point = ((start[0] + end[0]) // 2, (start[1] + end[1]) // 2)
        cv2.putText(
            app_globals.img,
            f"{distance:.2f} mm",
            mid_point,
            cv2.FONT_HERSHEY_SIMPLEX,
            0.5,
            (0, 255, 0),
            1,
            cv2.LINE_AA,
        )

    cv2.imshow("Image", app_globals.img)


def main() -> None:
    """Run the GUI interface."""
    # Parse command-line arguments
    parser = argparse.ArgumentParser(description="Image Measurement Tool")
    parser.add_argument("image_path", type=str, help="Path to the image file")
    args = parser.parse_args()

    # Load the image
    image_path: str = args.image_path
    app_globals.img = cv2.imread(image_path)
    if app_globals.img is None:
        print("Error: Image not found!")
        return

    app_globals.original_img = app_globals.img.copy()

    cv2.imshow("Image", app_globals.img)
    print("Instructions:")
    print("1. Left-click to set two points for the scale or measurement.")
    print("2. Enter the real-world size after setting the scale.")
    print("3. Measurements will be displayed on the image.")
    print("4. Right-click to remove the last measurement.")

    # Set mouse callback
    cv2.setMouseCallback("Image", handle_house_event)

    # Wait until the user closes the window
    cv2.waitKey(0)
    cv2.destroyAllWindows()


if __name__ == "__main__":
    main()
