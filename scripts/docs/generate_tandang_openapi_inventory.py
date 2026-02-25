#!/usr/bin/env python3
from __future__ import annotations

import json
import os
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def _now_utc_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def _load_json(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def _security_schemes(op: dict[str, Any]) -> list[str]:
    sec = op.get("security")
    if not sec:
        return []
    schemes: set[str] = set()
    for requirement in sec:
        if isinstance(requirement, dict):
            schemes.update(requirement.keys())
    return sorted(schemes)


def main() -> int:
    repo_root = Path(__file__).resolve().parents[2]
    default_spec = (
        repo_root
        / ".."
        / "tandang"
        / "markov-engine"
        / "contracts"
        / "openapi"
        / "api-v1.openapi.json"
    ).resolve()

    spec_path = Path(os.environ.get("TANDANG_OPENAPI_PATH", str(default_spec))).resolve()
    out_path = Path(
        os.environ.get(
            "TANDANG_OPENAPI_INVENTORY_OUT",
            str(repo_root / "docs" / "architecture" / "tandang-endpoints-openapi-inventory.md"),
        )
    ).resolve()

    spec = _load_json(spec_path)
    paths: dict[str, Any] = spec.get("paths", {})

    ops_by_tag: dict[str, list[dict[str, str]]] = defaultdict(list)
    for path, ops_map in paths.items():
        if not isinstance(ops_map, dict):
            continue
        for method, op in ops_map.items():
            if method.lower() not in {"get", "post", "put", "patch", "delete"}:
                continue
            if not isinstance(op, dict):
                continue
            tags = op.get("tags") or ["(untagged)"]
            tag = str(tags[0])
            summary = (op.get("summary") or "").strip()
            operation_id = (op.get("operationId") or "").strip()
            desc = summary or operation_id or ""
            schemes = _security_schemes(op)
            ops_by_tag[tag].append(
                {
                    "method": method.upper(),
                    "path": path,
                    "security": ", ".join(schemes) if schemes else "public/unspecified",
                    "desc": desc,
                }
            )

    lines: list[str] = []
    lines.append("# Tandang API — OpenAPI Inventory (Generated)\n")
    lines.append(f"Generated: `{_now_utc_iso()}`\n")
    lines.append(f"Source: `{spec_path}`\n")
    lines.append("")
    lines.append(
        "Notes:\n"
        "- This is generated from Tandang's OpenAPI contract.\n"
        "- Some endpoints' real auth behavior may differ from the contract; treat Tandang router middleware as source of truth.\n"
    )
    lines.append("")

    for tag in sorted(ops_by_tag.keys(), key=lambda t: (-len(ops_by_tag[t]), t.lower())):
        lines.append(f"## {tag}\n")
        lines.append("| Method | Path | Auth (per OpenAPI) | Summary |")
        lines.append("|---|---|---|---|")
        for op in sorted(ops_by_tag[tag], key=lambda o: (o["path"], o["method"])):
            method = op["method"]
            path = op["path"]
            security = op["security"]
            desc = op["desc"].replace("\n", " ").strip()
            lines.append(f"| `{method}` | `{path}` | `{security}` | {desc} |")
        lines.append("")

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")
    print(f"Wrote {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
