from pyduct.fittings import TwoWayFitting
from pyduct.fitting_types import elbow_round
from pyduct.connectors import Connector

# define objects
elbow_type = elbow_round(bend_radius=1, diameter=1, angle=130)
elbow = TwoWayFitting(name="elbow round", fitting_type=elbow_type)
elbow.connectors = [Connector(id="1", flowrate=5.0), Connector(id="2", flowrate=5.0)]

# calculate object
elbow.calculate()

for argument in ["name", "connectors", "fitting_type"]:
    value = str(elbow.__getattribute__(argument))
    print(f"{argument}: {value}")