import goblin

g = goblin.Object("mylib.dylib")
print(g.header)
print(g.name)

print("symbols")
globsym = None
start = -1
end = 9999999999

for sym in g.symbols():
    if "META" in sym.name and sym.is_global:
        if globsym is None:
            globsym = sym
            start = sym.meta.n_value

for sym in g.symbols():
    if sym.meta.n_value > start and sym.meta.n_value < end:
        print(f"sym after the found one: {sym}")
        end = sym.meta.n_value

print(f"found symbol {globsym.name} from {start} to {end}, size: {end-start}")
print(globsym)

print("libs")
print(g.libs)

print("rpaths")
print(g.rpaths)

print("exports")
print(len(g.exports()))

print("imports")
print(len(g.imports()))

print("sections")

for idx, section in enumerate(g.sections()):
    print(f"{idx+1}. {section}")
