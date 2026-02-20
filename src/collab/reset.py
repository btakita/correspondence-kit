"""
Regenerate template files in shared collaborator repos.

Rewrites AGENTS.md, README.md, CLAUDE.md symlink, .gitignore, voice.md,
and .github/workflows/notify.yml to match the current templates.

Usage:
  corrkit collab-reset alex          # Reset one collaborator
  corrkit collab-reset               # Reset all collaborators
"""

import argparse
import os
import shutil
import sys
from pathlib import Path

from . import load_collaborators
from .add import _generate_agents_md, _generate_readme_md

SHARED_DIR = Path("shared")
VOICE_FILE = Path("voice.md")
TEMPLATES_DIR = Path(__file__).parent / "templates"

_TEMPLATE_WORKFLOW = "notify.yml"


def _reset_one(name: str) -> None:
    """Regenerate template files for one collaborator."""
    sub_path = SHARED_DIR / name
    if not sub_path.exists():
        print(f"  {name}: submodule not found at shared/{name} -- skipping")
        return

    print(f"Resetting {name}...")

    # AGENTS.md
    (sub_path / "AGENTS.md").write_text(_generate_agents_md(name), encoding="utf-8")
    print("  Updated AGENTS.md")

    # CLAUDE.md symlink
    claude_md = sub_path / "CLAUDE.md"
    if claude_md.exists() or claude_md.is_symlink():
        claude_md.unlink()
    os.symlink("AGENTS.md", claude_md)
    print("  Updated CLAUDE.md -> AGENTS.md")

    # README.md
    (sub_path / "README.md").write_text(_generate_readme_md(name), encoding="utf-8")
    print("  Updated README.md")

    # .gitignore
    (sub_path / ".gitignore").write_text(
        "AGENTS.local.md\nCLAUDE.local.md\n__pycache__/\n", encoding="utf-8"
    )
    print("  Updated .gitignore")

    # voice.md
    if VOICE_FILE.exists():
        shutil.copy2(VOICE_FILE, sub_path / "voice.md")
        print("  Updated voice.md")

    # .github/workflows/notify.yml
    workflow_src = TEMPLATES_DIR / _TEMPLATE_WORKFLOW
    if workflow_src.exists():
        workflow_dir = sub_path / ".github" / "workflows"
        workflow_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(workflow_src, workflow_dir / _TEMPLATE_WORKFLOW)
        print(f"  Updated .github/workflows/{_TEMPLATE_WORKFLOW}")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Regenerate template files in shared collaborator repos"
    )
    parser.add_argument("name", nargs="?", help="Collaborator name (default: all)")
    args = parser.parse_args()

    collabs = load_collaborators()
    if not collabs:
        print("No collaborators configured in collaborators.toml")
        return

    names = [args.name] if args.name else list(collabs.keys())
    for name in names:
        if name not in collabs:
            print(f"Unknown collaborator: {name}")
            sys.exit(1)

    for name in names:
        _reset_one(name)

    print("\nDone. Run 'corrkit collab-sync' to push changes to remote.")


if __name__ == "__main__":
    main()
