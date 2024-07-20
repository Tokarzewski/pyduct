
def calc_velocity(flowrate: float, area: float) -> float:
    if flowrate < 0:
        raise ValueError("Flowrate must be positive:", flowrate)
    if area <= 0:
        raise ValueError("Area must be positive:", area)
    
    return flowrate / area