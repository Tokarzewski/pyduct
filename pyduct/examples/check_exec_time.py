import time

startTime = time.time()

import pyduct.examples.example8_find_crititcal_path

x = time.time() - startTime

print("The script took {0} second.".format(round(x, 2)))
