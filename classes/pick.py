# make an even distribution of class sizes
import os


dir = "benchmarking"


step = 5000 # 500 bytes
sizes = set()

for file in os.scandir(dir):
	if file.path.endswith(".class"):
		stats = file.stat()
		size = int(stats.st_size / step)
		#print(file.name, size)
		if size not in sizes:
			sizes.add(size)
		else:
			print("Removed", file)
			os.remove(file)
	else:
		os.remove(file)

