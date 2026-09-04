#!/usr/bin/env python3
"""Assemble Wenta.CUIX - a partial CUIX package for ZWCAD 2021 that adds a
'Wenta' ribbon tab with buttons. Modeled 1:1 on ZWSOFT's own APP+.cuix
partial package (same XML schema, same package layout).

Usage: python make_cuix.py [outdir]
"""
import struct
import sys
import zipfile
import zlib
from datetime import datetime
from pathlib import Path

OUT = Path(sys.argv[1] if len(sys.argv) > 1 else "bin") / "Wenta.CUIX"

# ----------------------------------------------------------------------------
# 1. tiny PNG writer (no dependencies)
# ----------------------------------------------------------------------------

def _chunk(tag: bytes, data: bytes) -> bytes:
    return (struct.pack(">I", len(data)) + tag + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF))


def make_png(width: int, height: int, pixels) -> bytes:
    """pixels: list of rows, each a list of (r, g, b, a) tuples."""
    raw = b"".join(
        b"\x00" + b"".join(struct.pack("4B", *px) for px in row)
        for row in pixels)
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return (b"\x89PNG\r\n\x1a\n"
            + _chunk(b"IHDR", ihdr)
            + _chunk(b"IDAT", zlib.compress(raw, 9))
            + _chunk(b"IEND", b""))


TEAL = (0, 128, 128, 255)
TEAL_DARK = (0, 84, 84, 255)
GRAY = (96, 110, 128, 255)
WHITE = (255, 255, 255, 255)
BG = (255, 255, 255, 0)          # transparent background


def icon_duct(size=32):
    """Duct cross-section: teal rounded-ish rectangle outline + airflow line."""
    px = [[BG] * size for _ in range(size)]
    m = 5                           # margin
    w = size - 2 * m                # inner square size
    for r in range(size):
        for c in range(size):
            edge = (r in (m, m + w) and m <= c <= m + w) or \
                   (c in (m, m + w) and m <= r <= m + w)
            if edge:
                px[r][c] = TEAL
    # airflow: horizontal line through the middle
    mid = size // 2
    for c in range(m + 4, m + w - 3):
        px[mid][c] = TEAL_DARK
    return make_png(size, size, px)


def icon_panel(size=32):
    """Palette panel: window with title bar."""
    px = [[BG] * size for _ in range(size)]
    m, w = 5, size - 2 * 5
    for r in range(m, m + w + 1):
        for c in range(m, m + w + 1):
            edge = r in (m, m + w) or c in (m, m + w)
            title = m < r <= m + 5 and m < c < m + w
            if edge:
                px[r][c] = GRAY
            elif title:
                px[r][c] = TEAL
    return make_png(size, size, px)


def icon_info(size=32):
    """Info: teal circle with a white 'i'."""
    px = [[BG] * size for _ in range(size)]
    cx = cy = (size - 1) / 2
    rad = size / 2 - 2
    for r in range(size):
        for c in range(size):
            d2 = (r - cx) ** 2 + (c - cy) ** 2
            if d2 <= rad * rad:
                px[r][c] = TEAL
    # the 'i': dot at (cy-5), stem cy-2..cy+5
    for c in (14, 15, 16):
        px[10][c] = px[11][c] = WHITE          # dot
    for r in range(14, 21):
        px[r][15] = WHITE                      # stem
    return make_png(size, size, px)


# ----------------------------------------------------------------------------
# 2. CUI XML parts
# ----------------------------------------------------------------------------

HEADER_CUI = """<?xml version="1.0"?>
<CustSection xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <FileVersion MajorVersion="0" MinorVersion="6" IncrementalVersion="1" UserVersion="1" />
  <Header>
    <CommonConfiguration>
      <CommonItems>
        <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
      </CommonItems>
    </CommonConfiguration>
  </Header>
</CustSection>"""

MENUGROUP_CUI = """<?xml version="1.0"?>
<MenuGroup xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" Name="Wenta" DisplayName="Wenta">
  <MacroGroup Name="WentaMacros">
    <MenuMacro UID="WENTA_DUCT">
      <Macro type="Any">
        <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
        <Name xlate="true" UID="WENTA_DUCT_NM">Duct Section</Name>
        <Command>^C^C_.WENTADUCT</Command>
        <HelpString xlate="true" UID="WENTA_DUCT_HS">Draw a rectangular duct cross-section and label it: WENTADUCT</HelpString>
        <SmallImage Name="" />
        <LargeImage Name="wenta_duct_32.png" />
      </Macro>
    </MenuMacro>
    <MenuMacro UID="WENTA_PANEL">
      <Macro type="Any">
        <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
        <Name xlate="true" UID="WENTA_PANEL_NM">Wenta Panel</Name>
        <Command>^C^C_.WENTAPANEL</Command>
        <HelpString xlate="true" UID="WENTA_PANEL_HS">Show the dockable Wenta palette panel: WENTAPANEL</HelpString>
        <SmallImage Name="" />
        <LargeImage Name="wenta_panel_32.png" />
      </Macro>
    </MenuMacro>
    <MenuMacro UID="WENTA_INFO">
      <Macro type="Any">
        <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
        <Name xlate="true" UID="WENTA_INFO_NM">Plugin Info</Name>
        <Command>^C^C_.WENTAHELLO</Command>
        <HelpString xlate="true" UID="WENTA_INFO_HS">Show Wenta plugin info: WENTAHELLO</HelpString>
        <SmallImage Name="" />
        <LargeImage Name="wenta_info_32.png" />
      </Macro>
    </MenuMacro>
    <MenuMacro UID="WENTA_CUCT">
      <Macro type="Any">
        <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
        <Name xlate="true" UID="WENTA_CUCT_NM">Fitting Catalog</Name>
        <Command>^C^C_.WENTACATALOG</Command>
        <HelpString xlate="true" UID="WENTA_CUCT_HS">Load the open zeta-catalog and show a lookup: WENTACATALOG</HelpString>
        <SmallImage Name="" />
        <LargeImage Name="wenta_info_32.png" />
      </Macro>
    </MenuMacro>
    <MenuMacro UID="WENTA_BOM">
      <Macro type="Any">
        <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
        <Name xlate="true" UID="WENTA_BOM_NM">BOM + KNR</Name>
        <Command>^C^C_.WENTABOM</Command>
        <HelpString xlate="true" UID="WENTA_BOM_HS">Solve the reference network and export the BOM with KNR rows: WENTABOM</HelpString>
        <SmallImage Name="" />
        <LargeImage Name="wenta_panel_32.png" />
      </Macro>
    </MenuMacro>
  </MacroGroup>
</MenuGroup>"""

RIBBONROOT_CUI = """<?xml version="1.0"?>
<RibbonRoot>
  <RibbonPanelSourceCollection xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
    <RibbonPanelSource UID="WENTA_RBPS" Text="Wenta" HiddenInEditor="false" KeyTip="WE">
      <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
      <Alias>ID_WentaPanel</Alias>
      <Name xlate="true" UID="WENTA_RBPS_NM">Wenta</Name>
      <DialogBoxLauncher UID="WENTA_RBPS_DBLR" CommandID="" CommandType="Macro">
        <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
      </DialogBoxLauncher>
      <RibbonRow UID="WENTA_RBRW">
        <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
        <RibbonCommandButton UID="WENTA_RBTN_DUCT" Id="ZwRibbonCommandButton" Text="Duct Section" ButtonStyle="LargeWithText" MenuMacroID="WENTA_DUCT" KeyTip="">
          <TooltipTitle xlate="true" UID="WENTA_RBTN_DUCT_TT">Duct Section</TooltipTitle>
          <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
        </RibbonCommandButton>
        <RibbonCommandButton UID="WENTA_RBTN_PANEL" Id="ZwRibbonCommandButton" Text="Wenta Panel" ButtonStyle="LargeWithText" MenuMacroID="WENTA_PANEL" KeyTip="">
          <TooltipTitle xlate="true" UID="WENTA_RBTN_PANEL_TT">Wenta Panel</TooltipTitle>
          <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
        </RibbonCommandButton>
        <RibbonCommandButton UID="WENTA_RBTN_INFO" Id="ZwRibbonCommandButton" Text="Plugin Info" ButtonStyle="LargeWithText" MenuMacroID="WENTA_INFO" KeyTip="">
          <TooltipTitle xlate="true" UID="WENTA_RBTN_INFO_TT">Plugin Info</TooltipTitle>
          <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
        </RibbonCommandButton>
        <RibbonCommandButton UID="WENTA_RBTN_CUCT" Id="ZwRibbonCommandButton" Text="Fitting Catalog" ButtonStyle="LargeWithText" MenuMacroID="WENTA_CUCT" KeyTip="">
          <TooltipTitle xlate="true" UID="WENTA_RBTN_CUCT_TT">Fitting Catalog</TooltipTitle>
          <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
        </RibbonCommandButton>
        <RibbonCommandButton UID="WENTA_RBTN_BOM" Id="ZwRibbonCommandButton" Text="BOM + KNR" ButtonStyle="LargeWithText" MenuMacroID="WENTA_BOM" KeyTip="">
          <TooltipTitle xlate="true" UID="WENTA_RBTN_BOM_TT">BOM + KNR</TooltipTitle>
          <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
        </RibbonCommandButton>
      </RibbonRow>
      <RibbonPanelBreak UID="WENTA_RPBRK" Id="ZwRibbonPanelBreak">
        <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
      </RibbonPanelBreak>
    </RibbonPanelSource>
  </RibbonPanelSourceCollection>
  <RibbonTabSourceCollection>
    <RibbonTabSource Text="Wenta" UID="WENTA_RBTS" DisplayType="Full" DefaultDisplay="AddToWorkSpace" WorkspaceBehavior="MergeOrAddTab">
      <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
      <Name xlate="true" UID="WENTA_RBTS_NM">Wenta</Name>
      <RibbonPanelSourceReference UID="WENTA_RBPSREF" PanelId="WENTA_RBPS" ResizeStyle="Default">
        <ModifiedRev MajorVersion="0" MinorVersion="0" UserVersion="0" />
      </RibbonPanelSourceReference>
    </RibbonTabSource>
  </RibbonTabSourceCollection>
</RibbonRoot>"""

STUBS = {
    "AcceleratorRoot.cui": '<AcceleratorRoot xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" />',
    "ImageMenuRoot.cui": '<ImageMenuRoot xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" />',
    "LSPFiles.cui": '<LSPFiles xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" />',
    "PopMenuRoot.cui": '<PopMenuRoot xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" />',
    "PopMenuRoot_Documentless.cui": '<PopMenuRoot_Documentless xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" />',
    "QuickAccessToolbarRoot.cui": '<QuickAccessToolbarRoot xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" />',
    "ToolbarRoot.cui": '<ToolbarRoot xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" />',
    "WorkspaceRoot.cui": '<WorkspaceRoot xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">\n  <WorkspaceConfigRoot />\n</WorkspaceRoot>',
}

CONTENT_TYPES = ('<?xml version="1.0" encoding="utf-8"?>'
                 '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
                 '<Default Extension="png" ContentType="text/xml" />'
                 '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml" />'
                 '<Default Extension="cui" ContentType="text/xml" />'
                 '<Default Extension="xml" ContentType="text/xml" />'
                 '</Types>')

IMAGES = {
    "wenta_duct_32.png": icon_duct(),
    "wenta_panel_32.png": icon_panel(),
    "wenta_info_32.png": icon_info(),
}


def rels_xml() -> str:
    rels = []
    i = 0

    def rel(target, rtype):
        nonlocal i
        i += 1
        rels.append(f'<Relationship Type="{rtype}" Target="/{target}" '
                    f'Id="R{i:016x}" />')

    for part in ["Header.cui", "WorkspaceRoot.cui", "MenuGroup.cui",
                 "RibbonRoot.cui"] + list(STUBS):
        rel(part, "CUI")
    rel("Menu_Package_Info.xml", "CUI")
    for img in IMAGES:
        rel(img, "Image")
    return ('<?xml version="1.0" encoding="utf-8"?>'
            '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            + "".join(rels) + "</Relationships>")


def package_info() -> str:
    now = datetime.now().isoformat()
    parts = ["Header.cui", "WorkspaceRoot.cui", "MenuGroup.cui",
             "RibbonRoot.cui", "Menu_Package_Info.xml"] + list(STUBS) + list(IMAGES)
    rows = "\n".join(
        f'  <PartData PartData_Name="/{p}" PartData_Modified="{now}" />'
        for p in parts)
    return ('<?xml version="1.0" encoding="utf-8"?>'
            "<MenuPackageParts>\n" + rows + "\n</MenuPackageParts>")


# ----------------------------------------------------------------------------
# 3. package it (STORED method, like ZWSOFT's own CUIX)
# ----------------------------------------------------------------------------

def main():
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(OUT, "w", zipfile.ZIP_STORED) as z:
        z.writestr("[Content_Types].xml", CONTENT_TYPES)
        z.writestr("_rels/.rels", rels_xml())
        z.writestr("Menu_Package_Info.xml", package_info())
        z.writestr("Header.cui", HEADER_CUI)
        z.writestr("MenuGroup.cui", MENUGROUP_CUI)
        z.writestr("RibbonRoot.cui", RIBBONROOT_CUI)
        for name, stub in STUBS.items():
            z.writestr(name, '<?xml version="1.0"?>\n' + stub)
        for name, png in IMAGES.items():
            z.writestr(name, png)
    print(f"Wenta.CUIX written: {OUT} ({OUT.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
