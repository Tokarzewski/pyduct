from pyductwork.fittings import TwoWayFitting
from pyductwork.fitting_types import elbow_round
from pyductwork.connectors import Connector

# define objects
elbow_type = elbow_round(name="elbow_round", bend_radius=1, diameter=1, angle=130)
elbow = TwoWayFitting(name="elbow", type=elbow_type)
elbow.connectors = [Connector(id="1", flowrate=5.0), Connector(id="2", flowrate=5.0)]

# calculate object
elbow.calculate()

for argument in ['name', 'connectors', 'type']:
    value = str(elbow.__getattribute__(argument))
    print(f"{argument}: {value}")