import sys

import apsw
import apsw.shell
import goblin

from dataclasses import dataclass
from enum import Flag, auto
from typing import Any, Callable, Iterator, Sequence, Tuple, cast

connection = apsw.Connection(":memory:")


@dataclass
class Generator:
    """A generator for the virtual table SQLite module.

    This class is needed because apsw wants to assign columns and
    column_access to the generator function itself."""

    columns: Sequence[str]
    column_access: apsw.ext.VTColumnAccess
    callable: Callable[[], Iterator[dict[str, Any]]]

    def __call__(self) -> Iterator[dict[str, Any]]:
        """Call the generator should return an iterator of dictionaries.

        The dictionaries should have keys that match the column names."""
        return self.callable()

    @staticmethod
    def make_generator(
        columns: list[str], generator: Callable[[], Iterator[dict[str, Any]]]
    ):
        """Create a generator from a callable that returns
        an iterator of dictionaries."""
        return Generator(columns, apsw.ext.VTColumnAccess.By_Name, generator)


class CacheFlag(Flag):
    NONE = 0
    DYNAMIC_ENTRIES = auto()
    HEADERS = auto()
    INSTRUCTIONS = auto()
    SECTIONS = auto()
    EXPORTS = auto()
    IMPORTS = auto()
    SYMBOLS = auto()
    STRINGS = auto()
    VERSION_REQUIREMENTS = auto()
    VERSION_DEFINITIONS = auto()
    DWARF_DIE = auto()
    DWARF_DIE_CALL_GRAPH = auto()

    @classmethod
    def from_string(cls, str: str):
        """Convert a string to a CacheFlag.

        This also specially handles 'ALL' which returns all the flags."""
        if str == "ALL":
            return cls.ALL()
        try:
            return cls[str]
        except KeyError:
            raise ValueError(f"{str} is not a valid CacheFlag")

    @classmethod
    def ALL(cls):
        retval = cls.NONE
        for member in cls.__members__.values():
            retval |= member
        return retval


def register_generator(
    connection: apsw.Connection,
    generator: Generator,
    table_name: str,
    generator_flag: CacheFlag,
    cache_flags: CacheFlag,
) -> None:
    """Register a virtual table generator.

    This method does a bit of duplicate work which checks if we need to cache
    the given generator.

    If so we rename the table with a prefix 'raw' and then create a temp table"""
    original_table_name = table_name
    if generator_flag in cache_flags:
        table_name = f"raw_{table_name}"

    apsw.ext.make_virtual_module(connection, table_name, generator)

    if generator_flag in cache_flags:
        connection.execute(
            f"""CREATE TABLE {original_table_name}
            AS SELECT * FROM {table_name};"""
        )


def register_symbols(
    gob: goblin.Object, connection: apsw.Connection, cache_flags: CacheFlag
) -> None:
    def dynamic_entries_generator() -> Iterator[dict[str, Any]]:
        for sym in gob.symbols():
            yield {
                "name": sym.name,
                "type": sym.typ,
                "global": sym.is_global,
                "weak": sym.weak,
                "undefined": sym.undefined,
                "stab": sym.stab,
            }

    generator = Generator.make_generator(
        ["name", "type", "global", "weak", "undefined", "stab"],
        dynamic_entries_generator,
    )

    register_generator(
        connection,
        generator,
        "macho_symbols",
        CacheFlag.SYMBOLS,
        cache_flags,
    )


def register_sections(
    gob: goblin.Object, connection: apsw.Connection, cache_flags: CacheFlag
) -> None:
    def dynamic_entries_generator() -> Iterator[dict[str, Any]]:
        for sect in gob.sections():
            yield {
                "name": sect.name,
                "segment": sect.segment,
                "addr": sect.addr,
                "size": sect.size,
                "offset": sect.offset,
                "align": sect.align,
                "reloff": sect.reloff,
                "nreloc": sect.nreloc,
                "flags": sect.flags,
            }

    generator = Generator.make_generator(
        [
            "name",
            "segment",
            "addr",
            "size",
            "offset",
            "align",
            "reloff",
            "nreloc",
            "flags",
        ],
        dynamic_entries_generator,
    )

    register_generator(
        connection,
        generator,
        "macho_sections",
        CacheFlag.SECTIONS,
        cache_flags,
    )


def register_exports(
    gob: goblin.Object, connection: apsw.Connection, cache_flags: CacheFlag
) -> None:
    def dynamic_entries_generator() -> Iterator[dict[str, Any]]:
        for exp in gob.exports():
            yield {
                "name": exp.name,
                "size": exp.size,
                "offset": exp.offset,
                "type": str(exp.info.typ),
                "address": exp.info.address,
                "flags": exp.info.flags,
                "lib": exp.info.lib,
                "lib_symbol_name": exp.info.lib_symbol_name,
            }

    generator = Generator.make_generator(
        [
            "name",
            "size",
            "offset",
            "type",
            "address",
            "flags",
            "lib",
            "lib_symbol_name",
        ],
        dynamic_entries_generator,
    )

    register_generator(
        connection,
        generator,
        "macho_exports",
        CacheFlag.EXPORTS,
        cache_flags,
    )


def register_imports(
    gob: goblin.Object, connection: apsw.Connection, cache_flags: CacheFlag
) -> None:
    def dynamic_entries_generator() -> Iterator[dict[str, Any]]:
        for imp in gob.imports():
            yield {
                "name": imp.name,
                "dylib": imp.dylib,
                "lazy": imp.is_lazy,
                "offset": imp.offset,
                "size": imp.size,
                "address": imp.address,
                "addend": imp.addend,
                "is_weak": imp.is_weak,
                "start_of_sequence_offset": imp.start_of_sequence_offset,
            }

    generator = Generator.make_generator(
        [
            "name",
            "dylib",
            "lazy",
            "offset",
            "size",
            "address",
            "addend",
            "is_weak",
            "start_of_sequence_offset",
        ],
        dynamic_entries_generator,
    )

    register_generator(
        connection,
        generator,
        "macho_imports",
        CacheFlag.IMPORTS,
        cache_flags,
    )


path = sys.argv[1]
g = goblin.Object(path)
register_symbols(g, connection, CacheFlag.SYMBOLS)
register_sections(g, connection, CacheFlag.SECTIONS)
register_exports(g, connection, CacheFlag.EXPORTS)
register_imports(g, connection, CacheFlag.IMPORTS)

shell = apsw.shell.Shell(db=connection, stdin=sys.stdin)
shell.command_prompt(["ölf> "])
shell.cmdloop()
