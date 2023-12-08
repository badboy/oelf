import goblin

g = goblin.Object("mylib.dylib")
print(g.header)
print(g.name)

print("symbols")
for sym in g.symbols():
    if not sym.undefined:
        print(sym)
        break

print("libs")
print(g.libs)

print("rpaths")
print(g.rpaths)

print("exports")
print(len(g.exports()))

print("imports")
print(len(g.imports()))

print("sections")
sections = iter(g.sections())
print(next(sections))
print(next(sections))
