# make an even distribution of class sizes
import os


dir = "benchmarking"


largeStep = int(100 * 1024)
smallStep = int(1.5 * 1024)
stepThreshold = 100000

largeSizes = set()
smallSizes = set()

for file in os.scandir(dir):
	try:
		if file.path.endswith(".class"):
			stats = file.stat()
			size = stats.st_size
			
			step = largeStep
			sizes = largeSizes
			if size < stepThreshold:
				step = smallStep
				sizes = smallSizes
			
			size = int(size / step)
			if size not in sizes:
				sizes.add(size)
			else:
				print("Removed", file)
				os.remove(file)
		else:
			os.remove(file)
	except:
		continue
