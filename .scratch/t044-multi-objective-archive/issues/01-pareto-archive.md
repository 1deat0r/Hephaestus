# 01: Multi-objective archive with Pareto frontier

**What to build:** `orchestrator::archive`: ValueBand, ArchiveNode
stamp, dominance test, bounded frontier with audited eviction.

**Status:** done

**Acceptance:**
- [x] AT-032: cheap-uncertain + expensive-plausible both stay (spec AC 1)
- [x] Dominated evictable, non-dominated never auto-evicted (spec AC 2)
- [x] Cap eviction audited (spec AC 3)
- [x] Twin-run byte-identical (spec AC 4)

## Comments
