# take stdin where each line is x,y and plot it

import sys
import matplotlib.pyplot as plt

x = []
y = []

for line in sys.stdin:
    x.append(float(line.split(',')[0]))
    y.append(float(line.split(',')[1]))

plt.plot(x, y)
plt.show()