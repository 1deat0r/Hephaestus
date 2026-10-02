# T-032 grill - independently qualified domain packs

Goal: independently qualified domain packs; a numerical/physical-science
pack specifies governing assumptions, units, simulator validity, equipment
authorization, appropriate human review; physical actuation separate from
ordinary code execution (IMPLEMENTATION_PLAN:112, R-102 continuity).

## Q1 - What is a domain pack (contract layer)?
**A:** A typed declaration: name+version, governing assumptions, unit
system, simulator validity conditions, equipment authorization list,
human-review requirements, and the qualification record (independently
qualified — capabilities do NOT inherit proof from the software domain,
M6 exit). The existing synthetic-evaluator is the software-domain pack;
this module types the CONTRACT + qualification gate. (agent-default)

## Q2 - Module placement + seam?
**A:** New `crates/hephaestus/src/domainpack/` module. Seam:
`DomainPack` record, `qualify(pack, checks) -> Result<QualifiedPack,
QualificationError>`, `validate_measurement(pack, quantity, unit) ->
Result<(), UnitError>`. (agent-default)

## Q3 - Required pack fields (roadmap)?
**A:** governing_assumptions (Vec), unit_system (named + dimension list),
simulator_validity (conditions under which the simulator may stand in for
reality), equipment_authorization (which equipment may be actuated/measured,
by whom), human_review (which decisions require it). (agent-default)

## Q4 - Independence of qualification (M6 exit)?
**A:** `qualify` requires: separate verifier identity (digest distinct from
the pack author digest), the pack's own oracle/evaluator qualified against
adjudicated fixtures (R-102 continuity), and scope-limited claims. A pack
whose verifier digest equals its author digest is refused (no
self-qualification). (agent-default)

## Q5 - Physical actuation separation (roadmap)?
**A:** The pack declares `physical_actuation: bool`; if true, its
execution path is typed SEPARATE from ordinary code execution
(ExecutionClass::PhysicalActuation vs CodeExecution) and the pack's
equipment authorization gates apply. A pack that declares code execution
but lists actuation equipment is refused. (agent-default)

## Q6 - Units (numerical pack)?
**A:** `validate_measurement` checks the declared unit against the pack's
unit system dimension list; an unknown dimension is a named UnitError.
(agent-default)

## Q7 - Simulator validity?
**A:** Conditions recorded as typed strings checked at measurement
validation: a measurement outside declared validity is refused
(simulation≠deployment continuity, §13). (agent-default)

## Q8 - Vocabulary?
**A:** GLOSSARY rows FIRST: Domain pack, Equipment authorization, Simulator
validity. Decision row before edit. (agent-default)

## Q9 - TDD seams?
**A:** Red-first per ticket: pack+qualification (01), measurement+actuation
separation (02). (agent-default)
