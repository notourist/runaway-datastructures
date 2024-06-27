import re

with open("hetzner.txt", "r") as f:
    csv_lines = ["name,time,build,read,space,overhead,bits"]
    for line in f.readlines():
        line = line.strip()
        if line.startswith("bits"):
            csv_lines[-1] = csv_lines[-1] + "," + str(line.split("=")[1])
        elif line.startswith("RESULT"):
            vs = []
            for kv in line.split(" ")[1:]:
                vs.append(kv.split("=")[1])
            csv_lines.append(",".join(vs))

with open("csv.txt", "w") as f:
    f.write("\n".join(csv_lines))
    f.write("\n")