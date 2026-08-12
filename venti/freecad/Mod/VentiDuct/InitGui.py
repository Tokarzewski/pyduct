# VentiDuct FreeCAD workbench entry point (loaded once by the FreeCAD GUI).
#
# This file is only imported inside FreeCAD; it registers the workbench that
# exposes the venti ductwork commands (sizing, solve, insulation, ...).

import FreeCADGui  # noqa: F401

from .commands import install_commands  # noqa: E402


class VentiDuctWorkbench(FreeCADGui.Workbench):
    MenuText = "Venti Ductwork"
    ToolTip = "Ductwork design — sizing, pressure drop, insulation, air balance (venti core)"
    Icon = ""
    Menu = ["&Ductwork"]
    Toolbar = ["VentiDuct"]

    def Initialize(self):
        from PySide2 import QtCore  # noqa: F401
        install_commands()
        cmds = ["VentiSize", "VentiSolve", "VentiInsulation"]
        self.appendMenu("&Ductwork", cmds)
        self.appendToolbar("VentiDuct", cmds)

    def GetClassName(self):
        return "Gui::PythonWorkbench"


FreeCADGui.addWorkbench("VentiDuctWorkbench")