from scipy.interpolate import CubicSpline, interp2d, interpn
from dataclasses import dataclass

@dataclass
class type1:
    name: str
    bend_radius: float
    diameter: float
    angle: float

    def dzeta(self):
        x_angle = [20, 30, 45, 60, 75, 90, 110, 130, 150, 180]
        y_RD = [0.50, 0.75, 1.00, 1.50, 2.00, 2.50]
        z_dzeta = [
            [0.22, 0.32, 0.43, 0.55, 0.64, 0.71, 0.8, 0.85, 0.91, 0.99],
            [0.1, 0.15, 0.2, 0.26, 0.3, 0.33, 0.37, 0.4, 0.42, 0.46],
            [0.07, 0.1, 0.13, 0.17, 0.2, 0.22, 0.25, 0.26, 0.28, 0.31],
            [0.05, 0.07, 0.09, 0.12, 0.14, 0.15, 0.17, 0.18, 0.19, 0.21],
            [0.04, 0.06, 0.08, 0.1, 0.12, 0.13, 0.15, 0.16, 0.17, 0.18],
            [0.04, 0.05, 0.07, 0.09, 0.11, 0.12, 0.14, 0.14, 0.15, 0.17],
        ]
        cs = interp2d(x_angle, y_RD, z_dzeta, kind="cubic")
        return float(cs(self.angle, self.bend_radius/self.diameter))

if __name__ == "__main__":
    print(type1('name', 1, 1, 130).dzeta())