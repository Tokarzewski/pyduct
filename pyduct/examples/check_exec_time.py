import time
import networkx as nx

startTime = time.time()

from example5_calculate_dp import dp_labels
print(dp_labels)
x = time.time() - startTime

print ('The script took {0} second.'.format(round(x,2)))