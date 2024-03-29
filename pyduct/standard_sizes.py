# units in mm
# increasing order of values

# EN 1505:2001 - Rectangular ducts

STANDARD_RECTANGULAR_DUCT_SIZES = [
    [100, 200], [150, 200], [200, 200], [100, 250], [150, 250],
    [200, 250], [250, 250], [100, 300], [150, 300], [200, 300],
    [250, 300], [300, 300], [100, 400], [150, 400], [200, 400],
    [250, 400], [300, 400], [400, 400], [150, 500], [200, 500],
    [250, 500], [300, 500], [400, 500], [500, 500], [150, 600],
    [200, 600], [250, 600], [300, 600], [400, 600], [500, 600],
    [600, 600], [200, 800], [250, 800], [300, 800], [400, 800],
    [500, 800], [600, 800], [800, 800], [250, 1000], [300, 1000],
    [400, 1000], [500, 1000], [600, 1000], [800, 1000], [1000, 1000],
    [300, 1200], [400, 1200], [500, 1200], [600, 1200], [800, 1200],
    [1000, 1200], [1200, 1200], [400, 1400], [500, 1400], [600, 1400],
    [800, 1400], [1000, 1400], [1200, 1400], [400, 1600], [500, 1600],
    [600, 1600], [800, 1600], [1000, 1600], [1200, 1600], [500, 1800],
    [600, 1800], [800, 1800], [1000, 1800], [1200, 1800], [500, 2000],
    [600, 2000], [800, 2000], [1000, 2000], [1200, 2000],
]

# EN 1506:2007 - Round ducts

STANDARD_ROUND_DUCT_SIZES = [
    63, 80, 100, 125, 150, 160, 200, 250, 300, 315, 355, 400,
    450, 500, 560, 630, 710, 800, 900, 1000, 1120, 1250,
]

# [d3, d1] => [d3, d1, d1]
STANDARD_ROUND_BRANCH_SIZES = [
    [63, 80], [80, 80], [63, 100], [80, 100],
    [100, 100], [80, 125], [100, 125], [125, 125],
    [80, 150], [100, 150], [125, 150], [150, 150],
    [80, 160], [100, 160], [125, 160], [150, 160],
    [160, 160], [80, 200], [100, 200], [125, 200],
    [150, 200], [160, 200], [200, 200], [80, 250],
    [100, 250], [125, 250], [150, 250], [160, 250],
    [200, 250], [250, 250], [100, 300], [125, 300],
    [150, 300], [160, 300], [200, 300], [250, 300],
    [300, 300], [100, 315], [125, 315], [150, 315],
    [160, 315], [200, 315], [250, 315], [300, 315],
    [315, 315], [160, 355], [200, 355], [250, 355],
    [300, 355], [315, 355], [355, 355], [160, 400],
    [200, 400], [250, 400], [300, 400], [315, 400],
    [355, 400], [400, 400], [200, 450], [250, 450],
    [300, 450], [315, 450], [355, 450], [400, 450],
    [450, 450], [200, 500], [250, 500], [300, 500],
    [315, 500], [355, 500], [400, 500], [450, 500],
    [500, 500], [250, 560], [300, 560], [315, 560],
    [355, 560], [400, 560], [450, 560], [500, 560],
    [560, 560], [250, 630], [300, 630], [315, 630],
    [355, 630], [400, 630], [450, 630], [500, 630],
    [560, 630], [630, 630], [315, 710], [355, 710],
    [400, 710], [450, 710], [500, 710], [560, 710],
    [630, 710], [710, 710], [315, 800], [355, 800],
    [400, 800], [450, 800], [500, 800], [560, 800],
    [630, 800], [710, 800], [800, 800], [400, 900],
    [450, 900], [500, 900], [560, 900], [630, 900],
    [710, 900], [800, 900], [900, 900], [400, 1000],
    [450, 1000], [500, 1000], [560, 1000], [630, 1000],
    [710, 1000], [800, 1000], [900, 1000], [1000, 1000],
    [500, 1120], [560, 1120], [630, 1120], [710, 1120],
    [800, 1120], [900, 1120], [1000, 1120], [1120, 1120],
    [500, 1250], [560, 1250], [630, 1250], [710, 1250],
    [800, 1250], [900, 1250], [1000, 1250], [1120, 1250],
    [1250, 1250]
]

# [d3, d1]
STANDARD_ROUND_TRANSFORMATION_SIZES = [
    [63, 80], [80, 80], [63, 100], [80, 100],
    [100, 100], [63, 125], [80, 125], [100, 125],
    [80, 150], [100, 150], [125, 150], [80, 160],
    [100, 160], [125, 160], [150, 160], [100, 200],
    [125, 200], [150, 200], [160, 200], [125, 250],
    [150, 250], [160, 250], [200, 250], [150, 300],
    [160, 300], [200, 300], [250, 300], [160, 315],
    [200, 315], [250, 315], [200, 355], [250, 355],
    [300, 355], [315, 355], [250, 400], [300, 400],
    [315, 400], [355, 400], [300, 450], [315, 450],
    [355, 450], [400, 450], [355, 500], [400, 500],
    [450, 500], [400, 560], [450, 560], [500, 560],
    [450, 630], [500, 630], [560, 630], [500, 710],
    [560, 710], [630, 710], [560, 800], [630, 800],
    [710, 800], [630, 900], [710, 900], [800, 900],
    [710, 1000], [800, 1000], [900, 1000], [800, 1120],
    [900, 1120], [1000, 1120], [900, 1250], [1000, 1250],
]
