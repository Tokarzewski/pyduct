import time

startTime = time.time()

from pyduct.examples.example7_calculate_dp import dp_labels
print(dp_labels)
x = time.time() - startTime

print ('The script took {0} second.'.format(round(x,2)))