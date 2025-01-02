import cv2
import numpy as np
import argparse
from typing import Optional

# Global variables
points: list[tuple[int, int]] = []
scale_factor: Optional[float] = None
img: Optional[np.ndarray] = None
original_img: Optional[np.ndarray] = None
measurements: list[tuple[tuple[int, int], tuple[int, int], float]] = []  # Stores lines and their measurements

def draw_circle(event: int, x: int, y: int, flags: int, param: Optional[object]) -> None:
    global points, scale_factor, img, original_img, measurements

    if img is None or original_img is None:
        return

    # On left mouse button click, store the point
    if event == cv2.EVENT_LBUTTONDOWN:
        points.append((x, y))
        cv2.circle(img, (x, y), 5, (0, 0, 255), -1)

        if len(points) == 2:
            # Draw a line between two points
            cv2.line(img, points[0], points[1], (255, 0, 0), 2)

            if scale_factor is None:
                try:
                    # First line sets the scale
                    print("Enter the real-world distance (in mm) of the line you just drew:")
                    real_distance: float = float(input())
                    pixel_distance: float = float(np.linalg.norm(np.array(points[0]) - np.array(points[1])))
                    scale_factor = real_distance / pixel_distance
                    print(f"Scale factor set to {scale_factor:.4f} mm per pixel")
                except ValueError:
                    print("Invalid input. Please enter a numeric value.")
                    points = []  # Reset points on invalid input
                except:
                    raise
            else:
                # Measure another line using the scale factor
                pixel_distance = float(np.linalg.norm(np.array(points[0]) - np.array(points[1])))
                real_distance = pixel_distance * scale_factor
                measurements.append((points[0], points[1], real_distance))
                print(f"Measured distance: {real_distance:.2f} mm")

            # Clear points after two clicks
            points = []
            redraw_image()

    elif event == cv2.EVENT_RBUTTONDOWN:
        # Remove the most recent measurement on right-click
        if measurements:
            measurements.pop()
            print("Last measurement removed.")
            redraw_image()

def redraw_image() -> None:
    global img, original_img, measurements

    if original_img is None:
        return

    # Reset to the original image
    img = original_img.copy()

    # Redraw all measurements
    for (start, end, distance) in measurements:
        cv2.line(img, start, end, (255, 0, 0), 2)
        mid_point = ((start[0] + end[0]) // 2, (start[1] + end[1]) // 2)
        cv2.putText(
            img,
            f"{distance:.2f} mm",
            mid_point,
            cv2.FONT_HERSHEY_SIMPLEX,
            0.5,
            (0, 255, 0),
            1,
            cv2.LINE_AA
        )

    cv2.imshow('Image', img)

def main() -> None:
    global img, original_img

    # Parse command-line arguments
    parser = argparse.ArgumentParser(description="Image Measurement Tool")
    parser.add_argument("image_path", type=str, help="Path to the image file")
    args = parser.parse_args()

    # Load the image
    image_path: str = args.image_path
    img = cv2.imread(image_path)
    if img is None:
        print("Error: Image not found!")
        return

    original_img = img.copy()

    cv2.imshow('Image', img)
    print("Instructions:")
    print("1. Left-click to set two points for the scale or measurement.")
    print("2. Enter the real-world size after setting the scale.")
    print("3. Measurements will be displayed on the image.")
    print("4. Right-click to remove the last measurement.")

    # Set mouse callback
    cv2.setMouseCallback('Image', draw_circle)

    # Wait until the user closes the window
    cv2.waitKey(0)
    cv2.destroyAllWindows()

if __name__ == "__main__":
    main()
