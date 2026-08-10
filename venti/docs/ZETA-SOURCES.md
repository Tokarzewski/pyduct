# Sources of duct-fitting loss coefficients (ζ)

A curated bibliography of authoritative sources for fitting pressure-loss
coefficients, for grounding and expanding `venti::components::fittings_library`.
Tier 1 are the primary references used by HVAC design software; Tier 2 are the
classic encyclopaedic compendia; Tier 3 are open/quick references and
machine-readable data.

> **Three things every ζ must carry — pick them before encoding data:**
> 1. **Reference velocity** — the ζ is only meaningful with the velocity it is
>    referred to (inlet / outlet / branch / main). `venti` already documents
>    this per correlation.
> 2. **Reynolds / size dependence** — the ASHRAE/SMACNA fitting data for round
>    elbows & tees in particular is *not* a constant; it is a function of Re and
>    duct size (the DB fits `ζ = C₀ + C₁·Re^C₂`-style curves). Decide whether to
>    store constants (design-tool level) or Re-corrected forms.
> 3. **Units** — most US sources are inch/IP; the coefficients themselves are
>    dimensionless but sizes/velocities in the *correlations behind them* are not.

---

## Tier 1 — Primary, used by commercial ductwork software

| Source | What it gives | Notes |
|---|---|---|
| **ASHRAE Handbook — Fundamentals** (Friction & Duct Design chapters, and *Applications* Ch.  "Duct Design") | friction + fitting loss coefficients; design velocity/ΔP guidance | The standard textbook reference; updated each 4-yr cycle. |
| **ASHRAE **Duct Fitting Database**** (the authoritative extra) | ~thousands of round/rectangular elbows, tees, transitions, dampers with loss coefficients as functions of geometry **and** Re | Licensed (published by ASHRAE; a Copt/CD/online database). This is what CADvent / Revit / MagiCAD actually use. Coefficients are keyed by fitting type + geometry parameters. |
| **SMACNA — HVAC Systems Duct Design** (and the **Duct Fitting Loss Coefficient** tables) | published tabulations of elbow/tee/transition ζ, plus duct construction standards | The sheet-metal contractors' standard; the most practical published tables. Companion: *HVAC Duct Construction Standards — Metal and Flexible*, and the **Duct Design Calculator**. |
| **CIBSE Guide B** (ex `B3, Ductwork`) & **ASHRAE/CIBSE** joint fitting data | UK-region duct design incl. fitting losses | UK counterpart to SMACNA; good for EN/Eurozone projects. |

## Tier 2 — Encyclopaedic compendia (deep, generic)

| Source | What it gives | Notes |
|---|---|---|
| **I. E. Idelchik, *Handbook of Hydraulic Resistance*** (CFD Press / CRC, 4th ed. 2007 is the English standard) | the largest generic compendium of loss coefficients — elbows, tees, transitions, orifices, valves, entries/exits — as charts and curves across Re and geometry | `venti` already cites Idelchik §6; the book is the deepest single source for *generic* (non-duct-specific) fittings. |
| **E. Fried & I. E. Idelchik, *Flow Resistance: A Design Guide for Engineers*** (CRC, 1989) | extracted, easy-to-use tables from Idelchik | condensed practical edition. |
| **D. S. Miller, *Internal Flow Systems*** (BHRA, 2nd ed. 1990) | pipe/duct fittings — tees, bends, valves — with loss data and worked design approach | strong on tees and on systematic phase/velocity treatment. |

## Tier 3 — Open / quick / machine-readable (good for validation & prototyping)

| Source | What it gives | Notes |
|---|---|---|
| **Engineering ToolBox** — *Duct Fittings / Pressure Loss* pages | quick elbow/tee/transition ζ tables and calculators | fast sanity checks, not a primary citation. |
| **Research regenerating the ASHRAE DB** (e.g. NREL/OSTI papers on "duct fitting loss coefficients" that genetic-program or curve-fit the ASHRAE database) | closed-form approximations reproducing the DB | open PDFs; useful to *derive* simplified formulas `venti` can ship without the licensed DB. |
| **Wikipedia**: *Darcy–Weisbach equation*, *Moody chart*, *Hydraulic diameter*, *ASHRAE Handbook*, *Idelchik* | background / interlinked authoritative definitions | not a ζ data source itself. |
| **Manufacturer tools** (Lindab, Beok, Ned Air, ALNOR) | product-specific ζ from real fittings (many free installers/calculators) | Wentyle (ALNOR) already ships multi-vendor fitting DBs; product catalogs are the practical end-use ζ. |

---

## Recommended path for `venti`

1. **Keep algebraic, documented correlations** (current approach) sourced from
   Idelchik / SMACNA / ASHRAE Fundamentals — good, transparent, no license.
2. **Add Re/size-corrected forms** for the Re-sensitive fittings (round elbow,
   tees) using *published* DB-regenerating formulas (Tier 3), falling back to
   Size-corrected constants.
3. **Build the catalog (FR-19)** schema from the Tier-1 **ASHRAE/SMACNA table
   structure** so a vendor's JSON catalogue maps onto it; keep `named_zeta` as
   the constant seed.
4. **Reference conventionally**: state the reference-velocity and Re basis in
   every doc comment (already the pattern in `venti`).

## Already cited by `pyduct`/`venti`
- ASHRAE Handbook — Fundamentals
- Swamee & Jain (1976) — explicit friction
- Colebrook–White — implicit friction
- Idelchik — Handbook of Hydraulic Resistance
- Hendiger, Ziętek, Chludzińska — *Wentylacja i Klimatyzacja…* (Polish reference, ALNOR/ZWCAD lineage, and the source Wentyle draws on)
