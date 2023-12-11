import sys
import oelf

path = "target/debug/liboelf.dylib"
if len(sys.argv) > 1:
    path = sys.argv[1]

g = oelf.Object(path)
print(f"{g.header=}")
print(f"{g.name=}")

print("symbols")
for symbol in g.symbols():
    print(symbol)

print("libs")
print(g.libs)

print("rpaths")
print(g.rpaths)

print("exports")
for export in g.exports():
    print(export)

print("imports")
for imp in g.imports():
    print(imp)

print("segments")
for segment in g.segments():
    print(f"{segment}")

print("sections")
for section in g.sections():
    print(f"{section}")

print("load commands")
for lcmd in g.load_commands():
    print(lcmd)
