# Zaka text and plain lists in gdb: text prints as the string of its octets,
# a plain list as its cells. Each is a structure of three words, `cells`,
# `count` and `room`, as the DWARF of zakac names them. A native build of
# zakac carries this script in `.debug_gdb_scripts`; `zakac debug-script gdb`
# prints it, for `source <file>`.
import gdb

PARTS = ["cells", "count", "room"]

# The most octets or cells a value shows, so a broken count reads no endless
# run.
MOST = 10000


def seq_kind(val):
    """The kind of a value of three words, or None for each other value."""
    kind = val.type.strip_typedefs()
    if kind.code != gdb.TYPE_CODE_STRUCT:
        return None
    if [f.name for f in kind.fields()] != PARTS:
        return None
    return kind


class ZakaText:
    """Text: the string of its `count` octets, read as UTF-8."""

    def __init__(self, val):
        self.val = val

    def to_string(self):
        count = min(int(self.val["count"]), MOST)
        if count == 0:
            return ""
        at = int(self.val["cells"])
        raw = gdb.selected_inferior().read_memory(at, count).tobytes()
        return raw.decode("utf-8", "replace")

    def display_hint(self):
        return "string"


class ZakaList:
    """A plain list: its `count` cells."""

    def __init__(self, val):
        self.val = val

    def to_string(self):
        return "%d cells" % int(self.val["count"])

    def children(self):
        cells = self.val["cells"]
        for n in range(min(int(self.val["count"]), MOST)):
            yield "[%d]" % n, (cells + n).dereference()

    def display_hint(self):
        return "array"


def lookup(val):
    """The printer of a Zaka value, or None for each other value."""
    kind = seq_kind(val)
    if kind is None:
        return None
    if kind.name == "text":
        return ZakaText(val)
    return ZakaList(val)


gdb.pretty_printers.append(lookup)
