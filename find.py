import os
import sys

IGNORE_DIRS = {"node_modules", "__pycache__", ".git", ".venv", ".pytest_cache", "dist", "build", ".angular", ".idea"}

def normalize(s: str) -> str:
    """Normalize string for matching: lowercase, strip spaces, underscores, and dashes."""
    return s.lower().replace(" ", "").replace("_", "").replace("-", "")

def walk_directory(base_dir, search_term=None, prefix="", show_all=True):
    try:
        entries = sorted(os.listdir(base_dir))
    except PermissionError:
        return False, []

    found_any = False
    lines = []
    norm_search = normalize(search_term) if search_term else None

    for i, entry in enumerate(entries):
        if entry in IGNORE_DIRS:
            continue
        path = os.path.join(base_dir, entry)
        connector = "└── " if i == len(entries) - 1 else "├── "

        # normalize entry for matching
        is_match = norm_search and norm_search in normalize(entry)

        if os.path.isdir(path):
            extension = "    " if i == len(entries) - 1 else "│   "
            child_found, child_lines = walk_directory(path, search_term, prefix + extension, show_all)
            if child_found:
                lines.append(prefix + connector + entry)
                lines.extend(child_lines)
                found_any = True
        else:
            if search_term:
                if is_match:
                    lines.append(prefix + connector + entry + "   <== match")
                    found_any = True
            elif show_all:
                lines.append(prefix + connector + entry)
                found_any = True

    return found_any, lines

def show_cwd_hierarchy(search_term):
    cwd = os.getcwd()
    parts = cwd.strip(os.sep).split(os.sep)
    if cwd.startswith(os.sep):
        parts = ["/"] + parts

    print("Project structure:")
    prefix = ""
    for i, part in enumerate(parts):
        connector = "└── " if i == len(parts) - 1 else "├── "
        marker = "   <== WORKDIR" if i == len(parts) - 1 else ""
        print(prefix + connector + part + marker)
        prefix += "    "

    found_any, lines = walk_directory(".", search_term, prefix, show_all=(search_term is None))
    for line in lines:
        print(line)

if __name__ == "__main__":
    search_term = sys.argv[1] if len(sys.argv) > 1 else None
    show_cwd_hierarchy(search_term)