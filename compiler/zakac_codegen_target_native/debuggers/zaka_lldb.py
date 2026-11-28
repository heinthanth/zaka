# Zaka text and plain lists in lldb: text shows as the string of its octets,
# a plain list as its cells. Each is a structure of three words, `cells`,
# `count` and `room`, as the DWARF of zakac names them. Save the output of
# `zakac debug-script lldb` to a file, then load it with
# `command script import <file>`.
import lldb

# The most octets or cells a value shows, so a broken count reads no endless
# run.
MOST = 10000


def part(val, name):
    """The number in one of the three words of a value."""
    return val.GetChildMemberWithName(name).GetValueAsUnsigned()


def text_line(val, _dict):
    """Text: the string of its `count` octets, read as UTF-8."""
    count = min(part(val, "count"), MOST)
    if count == 0:
        return '""'
    err = lldb.SBError()
    raw = val.GetProcess().ReadMemory(part(val, "cells"), count, err)
    if not err.Success():
        return "<unreadable>"
    return '"' + raw.decode("utf-8", "replace") + '"'


class ListCells:
    """A plain list: its `count` cells."""

    def __init__(self, val, _dict):
        self.val = val
        self.update()

    def update(self):
        cells = self.val.GetChildMemberWithName("cells")
        self.count = min(part(self.val, "count"), MOST)
        self.at = cells.GetValueAsUnsigned()
        self.cell = cells.GetType().GetPointeeType()
        return False

    def num_children(self):
        return self.count

    def get_child_index(self, name):
        try:
            return int(name.strip("[]"))
        except ValueError:
            return -1

    def get_child_at_index(self, n):
        at = self.at + n * self.cell.GetByteSize()
        return self.val.CreateValueFromAddress("[%d]" % n, at, self.cell)

    def has_children(self):
        return True


def __lldb_init_module(debugger, _dict):
    # zakac names text `text` and a plain list `<cell>[]`.
    here = __name__
    debugger.HandleCommand("type summary add -F %s.text_line text" % here)
    debugger.HandleCommand('type synthetic add -x "\\[\\]$" -l %s.ListCells' % here)
