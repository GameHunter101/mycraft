import matplotlib.pyplot as plt
import numpy as np

def chunk(l,n):
    for i in range(0,len(l),n):
        yield l[i:i+n]

file = open("./multithreading analysis/data.csv", "r")
lines = file.readlines()

threads = []
work_count = []
nanoseconds = []

for line in lines:
    separated = line.split(",")
    threads.append(int(separated[0]))
    work_count.append(int(separated[1]))
    escapes = "".join([chr(char) for char in range(1,32)])
    nanoseconds.append(int(separated[2].translate(escapes)))

fig = plt.figure()
ax = fig.add_subplot(projection="3d")

useable_data = []

chunks = list(chunk(nanoseconds,3))
for i in range((int)(threads.__len__()/3)):
    times = chunks[i]

    average = sum(times)/3
    useable_data.append((threads[i*3],work_count[i*3],average))

for i in useable_data:
    ax.scatter(i[0], i[1], i[2], marker="o")

ax.set_xlabel("Thread count")
ax.set_ylabel("Work count")
ax.set_zlabel("Time (nanoseconds)")

ax.set_xlim3d(1,12)
ax.set_ylim3d(1000,2000)
ax.set_zlim3d(0,150_000_000)

plt.show()