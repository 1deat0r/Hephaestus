#!/usr/bin/env python3
"""Generate strongly typed Rust contract records from schemas/contracts.schema.json.

T-002 deliverable (Hephaestus, IMPLEMENTATION_PLAN). Usage:

    python3 tools/generate_contracts_rs.py            # write generated.rs
    python3 tools/generate_contracts_rs.py --check    # exit 1 on drift (used by `make ci`)

The output is committed. Never edit crates/hephaestus/src/contracts/generated.rs by hand.
Stdlib only, so the generator runs without the validation venv.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "schemas" / "contracts.schema.json"
OUT_PATH = ROOT / "crates" / "hephaestus" / "src" / "contracts" / "generated.rs"

RUST_KEYWORDS = {
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
    "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
    "true", "type", "unsafe", "use", "where", "while", "async", "await", "dyn",
}


def pascal(segment: str) -> str:
    parts = [p for p in segment.replace("-", "_").split("_") if p]
    return "".join(p[:1].upper() + p[1:] for p in parts) or "Anon"


def variant_ident(value: str) -> str:
    ident = re.sub(r"[^A-Za-z0-9_]", "_", pascal(value))
    if not ident or ident[0].isdigit():
        ident = "V" + ident
    return ident


def rust_field(name: str) -> str:
    return "r#" + name if name in RUST_KEYWORDS else name


class StructField:
    def __init__(self, name: str, schema: dict, ty: str, required: bool):
        self.name = name
        self.schema = schema
        self.ty = ty
        self.required = required


class StructDef:
    def __init__(self, name: str, node: dict, path: str, is_record: bool):
        self.name = name
        self.node = node
        self.path = path
        self.is_record = is_record
        self.fields: list[StructField] = []


class EnumDef:
    def __init__(self, name: str, values: list, path: str):
        self.name = name
        self.values = values
        self.path = path


class Generator:
    def __init__(self, schema: dict):
        self.schema = schema
        self.defs = schema["$defs"]
        self.record_order = [s["$ref"].split("/")[-1] for s in schema["oneOf"]]
        self.structs: dict[str, StructDef] = {}
        self.struct_order: list[str] = []
        self.enums: dict[str, EnumDef] = {}
        self.enum_order: list[str] = []
        self.canon_to_name: dict[str, str] = {}
        self.pattern_order: list[str] = []

    # ---------- resolution & naming ----------

    def resolve(self, node: dict) -> tuple[dict, str | None]:
        if "$ref" in node:
            ref = node["$ref"]
            name = ref.split("/")[-1]
            if name not in self.defs:
                raise SystemExit(f"unresolvable $ref: {ref}")
            return self.defs[name], name
        return node, None

    def node_name(self, path: tuple[str, ...], node: dict) -> str:
        tail = path[-1] if path else "anon"
        if tail == "kind":
            return pascal(path[0]) + "Kind"
        if tail == "schema_version":
            return "SchemaVersion"
        if tail == "trust_origin":
            return "TrustOrigin"
        if tail == "provenance":
            return "Provenance"
        segments = [s for s in path if s != "items"]
        return pascal("_".join(segments))

    def register(self, canon: str, name: str, make) -> str:
        if canon in self.canon_to_name:
            existing = self.canon_to_name[canon]
            if existing != name:
                # Same shape reached under a second preferred/context name: reuse
                # the first name. Contexts where names collide with a DIFFERENT
                # shape are caught by the branch below.
                return existing
            return existing
        if name in self.structs or name in self.enums:
            raise SystemExit(f"duplicate type name with different shape: {name}")
        self.canon_to_name[canon] = name
        make(name)
        return name

    # ---------- type mapping ----------

    def rust_type(self, node: dict, path: tuple[str, ...]) -> str:
        raw, ref_name = self.resolve(node)
        if ref_name == "money":
            return "Money"
        if ref_name == "record_ref":
            return "RecordRef"

        if "anyOf" in raw:
            parts = raw["anyOf"]
            nulls = [p for p in parts if p.get("type") == "null"]
            if len(parts) != 2 or len(nulls) != 1:
                raise SystemExit(f"unsupported anyOf at {path}: {parts}")
            inner = next(p for p in parts if p.get("type") != "null")
            return f"Option<{self.rust_type(inner, path)}>"

        canon = json.dumps(raw, sort_keys=True)

        if "const" in raw:
            const = raw["const"]
            if isinstance(const, bool):
                return "bool"
            if isinstance(const, str):
                name = self.node_name(path, raw)

                def make(n, const=const, path=path):
                    self.enums[n] = EnumDef(n, [const], "_".join(path))
                    self.enum_order.append(n)

                return self.register(canon, name, make)
            raise SystemExit(f"unsupported const type at {path}: {const!r}")

        if "enum" in raw:
            values = raw["enum"]
            if not all(isinstance(v, str) for v in values):
                raise SystemExit(f"non-string enum at {path}: {values}")
            name = self.node_name(path, raw)

            def make(n, values=values, path=path):
                self.enums[n] = EnumDef(n, list(values), "_".join(path))
                self.enum_order.append(n)

            return self.register(canon, name, make)

        t = raw.get("type")
        if t == "object":
            name = self.node_name(path, raw)

            def make(n, raw=raw, path=path):
                self.structs[n] = StructDef(n, raw, "_".join(path), False)
                self.struct_order.append(n)

            return self.register(canon, name, make)
        if t == "array":
            if "items" not in raw:
                raise SystemExit(f"array without items at {path}")
            inner = self.rust_type(raw["items"], path + ("items",))
            return f"Vec<{inner}>"
        if t == "string":
            return "String"
        if t == "integer":
            return "i64"
        if t == "number":
            return "f64"
        if t == "boolean":
            return "bool"
        raise SystemExit(f"unsupported schema node at {path}: {json.dumps(raw)[:200]}")

    # ---------- constraints ----------

    def constraints(self, node: dict) -> list[tuple[str, object]]:
        raw, _ = self.resolve(node)
        if "anyOf" in raw:
            inner = next(p for p in raw["anyOf"] if p.get("type") != "null")
            raw, _ = self.resolve(inner)
        out: list[tuple[str, object]] = []
        for key in ("minLength", "maxLength", "minItems", "minimum", "maximum",
                    "exclusiveMinimum", "exclusiveMaximum"):
            if key in raw:
                out.append((key, raw[key]))
        if "pattern" in raw:
            out.append(("pattern", raw["pattern"]))
        if raw.get("format") == "date-time":
            out.append(("format", "date-time"))
        if raw.get("uniqueItems"):
            out.append(("uniqueItems", True))
        if isinstance(raw.get("const"), bool):
            out.append(("constBool", raw["const"]))
        return out

    # ---------- collection ----------

    def collect(self) -> None:
        # Shared value objects referenced via $ref (rust_type short-circuits on
        # them, so register their defs explicitly here).
        for special in ("record_ref", "money"):
            self.rust_type(self.defs[special], (special,))
        for kind in self.record_order:
            node = self.defs[kind]
            if node.get("additionalProperties") is not False:
                raise SystemExit(f"record {kind} must set additionalProperties:false")
            ty = self.rust_type(node, (kind,))
            if ty != pascal(kind):
                raise SystemExit(f"record {kind} mapped to {ty}")
            struct = self.structs[ty]
            struct.is_record = True
            for fname, fschema in node["properties"].items():
                required = fname in node["required"]
                fty = self.rust_type(fschema, (kind, fname))
                struct.fields.append(StructField(fname, fschema, fty, required))
        # Populate nested structs (phase 1 registers object nodes but does not
        # descend into their properties). Worklist: populating a struct may
        # register deeper types that need populating too.
        i = 0
        while i < len(self.struct_order):
            struct = self.structs[self.struct_order[i]]
            i += 1
            if struct.fields:
                continue
            parent = tuple(struct.path.split("_"))
            for fname, fschema in struct.node["properties"].items():
                fty = self.rust_type(fschema, parent + (fname,))
                required = fname in struct.node.get("required", [])
                struct.fields.append(StructField(fname, fschema, fty, required))
        for name in self.struct_order:
            node = self.structs[name].node
            if node.get("additionalProperties") is not False:
                raise SystemExit(f"nested struct {name} lacks additionalProperties:false")

    # ---------- rendering ----------

    @staticmethod
    def split_type(ty: str) -> tuple[str, str]:
        if ty.startswith("Option<"):
            return "option", ty[len("Option<"):-1]
        if ty.startswith("Vec<"):
            return "vec", ty[len("Vec<"):-1]
        return "plain", ty

    def pattern_static(self, pattern: str) -> str:
        if pattern not in self.pattern_order:
            self.pattern_order.append(pattern)
        return f"PATTERN_{self.pattern_order.index(pattern)}"

    @staticmethod
    def num_literal(cv: object, unit: str) -> str:
        if unit == "f64":
            return f"{json.dumps(cv)}f64" if not isinstance(cv, int) else f"{cv}.0f64"
        return f"{int(cv)}i64"

    def rules(self, expr: str, ty: str, cons: list, path_lit: str, by_ref: bool = True) -> list[str]:
        """Validation lines for a non-Option expression `expr` of type `ty`.

        `expr` must be usable by value for Copy types and by method-call for
        String/Vec/structs (works for both `self.f` and `v` from `if let Some(v)`).
        """
        out: list[str] = []
        if ty in self.structs:
            out.append(f"for v in {expr}.validate() {{")
            out.append(f'    out.push(ContractViolation {{ path: format!("{{}}.{{}}", {path_lit}, v.path), message: v.message }});')
            out.append("}")
            return out
        if ty == "String":
            for ck, cv in cons:
                if ck == "minLength":
                    out.append(f"if {expr}.chars().count() < {int(cv)} {{")
                    out.append(f'    out.push(ContractViolation {{ path: {path_lit}.to_string(), message: format!("minLength: expected >= {int(cv)}, got {{}}", {expr}.chars().count()) }});')
                    out.append("}")
                elif ck == "maxLength":
                    out.append(f"if {expr}.chars().count() > {int(cv)} {{")
                    out.append(f'    out.push(ContractViolation {{ path: {path_lit}.to_string(), message: format!("maxLength: expected <= {int(cv)}, got {{}}", {expr}.chars().count()) }});')
                    out.append("}")
                elif ck == "pattern":
                    static = self.pattern_static(cv)
                    borrow = "&" if by_ref else ""
                    out.append(f"if !{static}.get_or_init(|| Regex::new({json.dumps(cv)}).expect(\"generated pattern\")).is_match({borrow}{expr}) {{")
                    out.append(f'    out.push(ContractViolation {{ path: {path_lit}.to_string(), message: "pattern: {cv} failed".to_string() }});')
                    out.append("}")
                elif ck == "format":
                    borrow = "&" if by_ref else ""
                    out.append(f"if OffsetDateTime::parse({borrow}{expr}, &Rfc3339).is_err() {{")
                    out.append(f'    out.push(ContractViolation {{ path: {path_lit}.to_string(), message: "format: expected RFC 3339 date-time".to_string() }});')
                    out.append("}")
            return out
        if ty in ("i64", "f64"):
            for ck, cv in cons:
                if ck not in ("minimum", "maximum", "exclusiveMinimum", "exclusiveMaximum"):
                    continue
                op = {"minimum": "<", "maximum": ">", "exclusiveMinimum": "<=", "exclusiveMaximum": ">="}[ck]
                want = {"minimum": "expected >=", "maximum": "expected <=",
                        "exclusiveMinimum": "expected >", "exclusiveMaximum": "expected <"}[ck]
                out.append(f"if {expr} {op} ({self.num_literal(cv, ty)}) {{")
                out.append(f'    out.push(ContractViolation {{ path: {path_lit}.to_string(), message: "{ck}: {want} {cv}".to_string() }});')
                out.append("}")
            return out
        if ty == "bool":
            for ck, cv in cons:
                if ck == "constBool":
                    want = "true" if cv else "false"
                    cond = f"!{expr}" if cv else expr
                    out.append(f"if {cond} {{")
                    out.append(f'    out.push(ContractViolation {{ path: {path_lit}.to_string(), message: "const: expected {want}".to_string() }});')
                    out.append("}")
            return out
        if ty.startswith("Vec<"):
            inner = ty[len("Vec<"):-1]
            for ck, cv in cons:
                if ck == "minItems" and int(cv) > 0:
                    if int(cv) == 1:
                        out.append(f"if {expr}.is_empty() {{")
                    else:
                        out.append(f"if {expr}.len() < {int(cv)} {{")
                    out.append(f'    out.push(ContractViolation {{ path: {path_lit}.to_string(), message: format!("minItems: expected >= {int(cv)}, got {{}}", {expr}.len()) }});')
                    out.append("}")
                elif ck == "uniqueItems":
                    out.append("{")
                    out.append("    let mut seen = std::collections::HashSet::new();")
                    out.append(f"    for item in {expr}.iter() {{")
                    out.append("        if let Ok(v) = serde_json::to_value(item) && !seen.insert(v) {")
                    out.append(f'            out.push(ContractViolation {{ path: {path_lit}.to_string(), message: "uniqueItems: duplicate element".to_string() }});')
                    out.append("        }")
                    out.append("    }")
                    out.append("}")
            if inner in self.structs:
                out.append(f"for (i, item) in {expr}.iter().enumerate() {{")
                out.append("    for v in item.validate() {")
                out.append(f'        out.push(ContractViolation {{ path: format!("{{}}[{{}}].{{}}", {path_lit}, i, v.path), message: v.message }});')
                out.append("    }")
                out.append("}")
            return out
        return out

    def render_validate(self, struct: StructDef) -> list[str]:
        body: list[str] = []
        for f in struct.fields:
            cons = self.constraints(f.schema)
            if not cons and self.split_type(f.ty)[1] not in self.structs \
                    and not self.split_type(f.ty)[1].startswith("Vec<"):
                body.append(f"        // {f.name}")
                continue
            kind, base = self.split_type(f.ty)
            fname = rust_field(f.name)
            path_lit = f'"{f.name}"'
            body.append(f"        // {f.name}")
            if kind == "option":
                # Copy types must be dereferenced: std has no `PartialOrd<Rhs>
                # for &T` with an owned Rhs, and method-call forms autoderef.
                vexpr = "(*v)" if base in ("i64", "f64", "bool") else "v"
                inner = self.rules(vexpr, base, cons, path_lit, by_ref=False)
                if (len(inner) == 3 and inner[0].startswith("if ")
                        and inner[0].endswith(" {") and inner[2] == "}"):
                    # Single condition: emit an if-let chain so clippy's
                    # collapsible_if stays quiet (edition 2024 let-chains).
                    cond = inner[0][3:-2]
                    body.append(f"        if let Some(v) = &self.{fname} && {cond} {{")
                    body.append("            " + inner[1])
                    body.append("        }")
                else:
                    body.append(f"        if let Some(v) = &self.{fname} {{")
                    body.extend("            " + ln for ln in inner)
                    body.append("        }")
            else:
                expr = f"self.{fname}"
                body.extend("        " + ln for ln in self.rules(expr, f.ty, cons, path_lit))
        bind = "let mut out: Vec<ContractViolation> = Vec::new();" \
            if any("out.push" in ln for ln in body) \
            else "let out: Vec<ContractViolation> = Vec::new();"
        return [
            "    /// Structural deserialization already enforces types, required fields,",
            "    /// enums and const strings. This adds format, pattern, bound and",
            "    /// uniqueness checks (T-002 contract validation).",
            "    pub fn validate(&self) -> Vec<ContractViolation> {",
            f"        {bind}",
            *body,
            "        out",
            "    }",
        ]

    def render_struct(self, struct: StructDef) -> list[str]:
        lines: list[str] = []
        if struct.is_record:
            lines.append(f"/// `{struct.path}` record (contracts schema 1.2).")
        else:
            lines.append(f"/// Nested contract object `{struct.path}`.")
        lines.append("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]")
        lines.append("#[serde(deny_unknown_fields)]")
        lines.append(f"pub struct {struct.name} {{")
        for f in struct.fields:
            if f.name != "id" or struct.is_record:
                lines.append(f"    #[serde(rename = {json.dumps(f.name)})]")
            if not f.required:
                lines.append('    #[serde(skip_serializing_if = "Option::is_none")]')
            lines.append(f"    pub {rust_field(f.name)}: {f.ty},")
        lines.append("}")
        lines.append("")
        lines.append(f"impl {struct.name} {{")
        lines.extend(self.render_validate(struct))
        if struct.is_record:
            lines.append("")
            lines.append("    pub fn id(&self) -> &str {")
            lines.append("        &self.id")
            lines.append("    }")
            lines.append("")
            lines.append("    pub fn record_version(&self) -> i64 {")
            lines.append("        self.record_version")
            lines.append("    }")
        lines.append("}")
        lines.append("")
        return lines

    def render_enum(self, e: EnumDef) -> list[str]:
        variants: list[tuple[str, str]] = []
        seen: set[str] = set()
        for value in e.values:
            ident = variant_ident(value)
            if ident in seen:
                raise SystemExit(f"variant collision in {e.name}: {value}")
            seen.add(ident)
            variants.append((ident, value))
        lines = [
            f"/// `{e.path}` ({', '.join(json.dumps(v) for v in e.values)}).",
            "#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]",
            f"pub enum {e.name} {{",
        ]
        for ident, value in variants:
            lines.append(f"    #[serde(rename = {json.dumps(value)})]")
            lines.append(f"    {ident},")
        lines.append("}")
        lines.append("")
        return lines

    def render_contract_record(self) -> list[str]:
        lines = [
            "/// One instance of any principal contract record (schema 1.2).",
            "/// Deserialize via [`ContractRecord::from_value`], which rejects",
            "/// unsupported schema versions instead of guessing their meaning.",
            "#[derive(Debug, Clone, PartialEq)]",
            "#[allow(clippy::large_enum_variant)] // records are transient values; boxing adds churn, not wins",
            "pub enum ContractRecord {",
        ]
        for kind in self.record_order:
            lines.append(f"    {pascal(kind)}({pascal(kind)}),")
        lines.append("}")
        lines.append("")
        lines.append("impl ContractRecord {")
        lines.append("    /// Parse a single record, enforcing the current schema version gate")
        lines.append("    /// and running generated contract validation.")
        lines.append("    pub fn from_value(value: serde_json::Value) -> Result<Self, ContractError> {")
        lines.append("        let version = value")
        lines.append('            .get("schema_version")')
        lines.append("            .and_then(serde_json::Value::as_str)")
        lines.append('            .ok_or_else(|| ContractError::MalformedRecord("missing schema_version".to_string()))?;')
        lines.append("        if version != SCHEMA_VERSION {")
        lines.append("            return Err(ContractError::UnsupportedSchemaVersion(version.to_string()));")
        lines.append("        }")
        lines.append("        let kind = value")
        lines.append('            .get("kind")')
        lines.append("            .and_then(serde_json::Value::as_str)")
        lines.append('            .ok_or_else(|| ContractError::MalformedRecord("missing kind".to_string()))?;')
        lines.append("        let record = match kind {")
        for kind in self.record_order:
            lines.append(f'            {json.dumps(kind)} => Self::{pascal(kind)}(')
            lines.append("                serde_json::from_value(value).map_err(ContractError::Json)?,")
            lines.append("            ),")
        lines.append('            other => return Err(ContractError::UnknownKind(other.to_string())),')
        lines.append("        };")
        lines.append("        let violations = record.validate();")
        lines.append("        if violations.is_empty() {")
        lines.append("            Ok(record)")
        lines.append("        } else {")
        lines.append("            Err(ContractError::Invalid(violations))")
        lines.append("        }")
        lines.append("    }")
        lines.append("")
        lines.append("    pub fn validate(&self) -> Vec<ContractViolation> {")
        lines.append("        match self {")
        for kind in self.record_order:
            lines.append(f"            Self::{pascal(kind)}(r) => r.validate(),")
        lines.append("        }")
        lines.append("    }")
        lines.append("")
        lines.append("    pub fn to_value(&self) -> Result<serde_json::Value, serde_json::Error> {")
        lines.append("        match self {")
        for kind in self.record_order:
            lines.append(f"            Self::{pascal(kind)}(r) => serde_json::to_value(r),")
        lines.append("        }")
        lines.append("    }")
        lines.append("")
        lines.append("    pub fn kind(&self) -> &'static str {")
        lines.append("        match self {")
        for kind in self.record_order:
            lines.append(f'            Self::{pascal(kind)}(_) => {json.dumps(kind)},')
        lines.append("        }")
        lines.append("    }")
        lines.append("")
        lines.append("    pub fn id(&self) -> &str {")
        lines.append("        match self {")
        for kind in self.record_order:
            lines.append(f"            Self::{pascal(kind)}(r) => r.id(),")
        lines.append("        }")
        lines.append("    }")
        lines.append("")
        lines.append("    pub fn record_version(&self) -> i64 {")
        lines.append("        match self {")
        for kind in self.record_order:
            lines.append(f"            Self::{pascal(kind)}(r) => r.record_version(),")
        lines.append("        }")
        lines.append("    }")
        lines.append("}")
        lines.append("")
        return lines

    def render(self) -> str:
        self.collect()
        schema_sha = hashlib.sha256(SCHEMA_PATH.read_bytes()).hexdigest()
        versions = {
            self.defs[k]["properties"]["schema_version"]["const"]
            for k in self.record_order
        }
        if versions != {"1.2"}:
            raise SystemExit(f"unexpected schema versions: {versions}")

        out: list[str] = [
            "// @generated by tools/generate_contracts_rs.py from schemas/contracts.schema.json.",
            f"// schema sha256: {schema_sha}. DO NOT EDIT BY HAND — run the generator.",
            "#![allow(clippy::too_many_lines)]",
            "",
            "use regex_lite::Regex;",
            "use serde::{Deserialize, Serialize};",
            "use std::sync::OnceLock;",
            "use time::format_description::well_known::Rfc3339;",
            "use time::OffsetDateTime;",
            "",
            "use super::{ContractError, ContractViolation};",
            "",
            "/// The single schema version this generated module implements.",
            'pub const SCHEMA_VERSION: &str = "1.2";',
            "",
        ]
        for name in self.enum_order:
            out.extend(self.render_enum(self.enums[name]))
        for name in self.struct_order:
            out.extend(self.render_struct(self.structs[name]))
        out.extend(self.render_contract_record())
        if self.pattern_order:
            out.append("// Compiled-once pattern checks for string constraints.")
            for i, pat in enumerate(self.pattern_order):
                out.append(f"// {pat}")
                out.append(f"static PATTERN_{i}: OnceLock<Regex> = OnceLock::new();")
            out.append("")
        return "\n".join(out) + "\n"


def rustfmt(text: str) -> str:
    """Format through rustfmt so `cargo fmt --check` and --check both pass."""
    import subprocess

    proc = subprocess.run(
        ["rustfmt", "--edition", "2024"],
        input=text,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        raise SystemExit(f"rustfmt failed on generated code:\n{proc.stderr}")
    return proc.stdout


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail if generated.rs is stale")
    args = parser.parse_args()

    schema = json.loads(SCHEMA_PATH.read_text())
    rendered = rustfmt(Generator(schema).render())

    if args.check:
        if not OUT_PATH.exists() or OUT_PATH.read_text() != rendered:
            print(
                "STALE: crates/hephaestus/src/contracts/generated.rs does not match the schema.\n"
                "Run: python3 tools/generate_contracts_rs.py",
                file=sys.stderr,
            )
            return 1
        print("generated.rs is up to date")
        return 0

    OUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUT_PATH.write_text(rendered)
    print(f"wrote {OUT_PATH.relative_to(ROOT)} ({len(rendered.splitlines())} lines)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
