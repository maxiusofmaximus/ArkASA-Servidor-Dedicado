#!/usr/bin/env python3
"""
ARK SA CurseForge Mod Scraper
──────────────────────────────
Paginates through all ARK: Survival Ascended mods (gameId=1172434) via the
CurseForge v1 API and writes the results to src-tauri/src/mods_db.json so
the Rust binary can embed them at compile time via include_str!().

Usage:
    python scripts/scrape_mods.py

Requirements:
    pip install requests

API key is read from:
    %APPDATA%\ARK ASA Config Manager\curseforge_api_key.txt

or from the environment variable:
    CURSEFORGE_API_KEY=<your_key>
"""

import json
import os
import sys
import time
from pathlib import Path

try:
    import requests
except ImportError:
    print("ERROR: 'requests' is not installed. Run:  pip install requests")
    sys.exit(1)

# ─── Config ──────────────────────────────────────────────────────────────────

ARK_ASA_GAME_ID = 1172434
BASE_URL = "https://api.curseforge.com/v1/mods/search"
PAGE_SIZE = 50           # max allowed by the API
DELAY_BETWEEN_PAGES = 0.4   # seconds — be polite to the API
OUTPUT_PATH = Path(__file__).parent.parent / "src-tauri" / "src" / "mods_db.json"

# ─── API Key ─────────────────────────────────────────────────────────────────

def load_api_key() -> str:
    # 1. Environment variable
    key = os.environ.get("CURSEFORGE_API_KEY", "").strip()
    if key:
        return key

    # 2. %APPDATA% config file
    appdata = os.environ.get("APPDATA", "")
    if appdata:
        key_file = Path(appdata) / "ARK ASA Config Manager" / "curseforge_api_key.txt"
        if key_file.exists():
            key = key_file.read_text(encoding="utf-8").strip()
            if key:
                return key

    print(
        "ERROR: No CurseForge API key found.\n"
        "  Option A — set env var:  set CURSEFORGE_API_KEY=<your_key>\n"
        "  Option B — paste your key into the app and click SAVE API KEY,\n"
        "             then re-run this script."
    )
    sys.exit(1)


# ─── Fetch helpers ────────────────────────────────────────────────────────────

def fetch_page(session: requests.Session, api_key: str, index: int) -> dict:
    params = {
        "gameId": ARK_ASA_GAME_ID,
        "sortField": 2,       # 2 = Popular
        "sortOrder": "desc",
        "pageSize": PAGE_SIZE,
        "index": index,
    }
    headers = {
        "x-api-key": api_key,
        "Accept": "application/json",
    }

    for attempt in range(3):
        try:
            resp = session.get(BASE_URL, params=params, headers=headers, timeout=15)
        except requests.RequestException as e:
            print(f"  Network error (attempt {attempt+1}/3): {e}")
            time.sleep(2 ** attempt)
            continue

        if resp.status_code == 429:
            wait = int(resp.headers.get("Retry-After", "5"))
            print(f"  Rate limited — waiting {wait}s …")
            time.sleep(wait)
            continue
        if resp.status_code == 403:
            print("ERROR: API key rejected (403 Forbidden). Check your key.")
            sys.exit(1)
        resp.raise_for_status()
        return resp.json()

    print("ERROR: Failed after 3 attempts.")
    sys.exit(1)


def cf_mod_to_record(m: dict) -> dict:
    """Convert a raw CurseForge mod object to the schema used by the Rust binary."""
    return {
        "id": str(m.get("id", "")),
        "name": m.get("name", ""),
        "summary": m.get("summary") or "",
        "download_count": m.get("downloadCount") or 0,
        "categories": [c["name"] for c in (m.get("categories") or [])],
        "logo_url": (m.get("logo") or {}).get("thumbnailUrl"),
        "slug": m.get("slug") or "",
    }


# ─── Main ─────────────────────────────────────────────────────────────────────

def main():
    api_key = load_api_key()
    session = requests.Session()

    print(f"Fetching ARK SA mods from CurseForge (gameId={ARK_ASA_GAME_ID}) …")

    all_mods: list[dict] = []
    index = 0
    total = None

    while True:
        print(f"  Page {index // PAGE_SIZE + 1}  (offset {index}) …", end=" ", flush=True)
        data = fetch_page(session, api_key, index)

        page_mods = data.get("data", [])
        if total is None:
            total = data.get("pagination", {}).get("totalCount", 0)
            print(f"total={total}")

        records = [cf_mod_to_record(m) for m in page_mods]
        all_mods.extend(records)
        print(f"fetched {len(records)} mods  (cumulative: {len(all_mods)})")

        if not page_mods or len(all_mods) >= total:
            break

        index += PAGE_SIZE
        time.sleep(DELAY_BETWEEN_PAGES)

    # Deduplicate by ID (shouldn't happen, but just in case)
    seen: set[str] = set()
    unique_mods = []
    for mod in all_mods:
        if mod["id"] not in seen:
            seen.add(mod["id"])
            unique_mods.append(mod)

    print(f"\nTotal unique mods scraped: {len(unique_mods)}")

    # Write output
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(OUTPUT_PATH, "w", encoding="utf-8") as f:
        json.dump(unique_mods, f, ensure_ascii=False, indent=2)

    print(f"Saved → {OUTPUT_PATH}")
    print("Done! Rebuild the Tauri app to embed the updated DB.")


if __name__ == "__main__":
    main()
