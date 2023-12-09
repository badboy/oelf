import oelf

g = oelf.Object("target/debug/liboelf.dylib")
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

print("sections")
for section in g.sections():
    print(f"{section}")
