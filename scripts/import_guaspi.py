#!/usr/bin/env python3
import csv
import json
import os
import sys
import argparse
import time
import re
from pathlib import Path

try:
    import urllib.request
    import urllib.error
except ImportError:
    print("urllib required.", file=sys.stderr)
    sys.exit(1)

# Configuration
API_URL_DEFAULT = "https://lensisku-dev.lojban.org/api"
TSV_FILE = "scripts/files/guaspi.tsv"

GUASPI_LANG_ID = 316
ENGLISH_LANG_ID = 2
CHINESE_LANG_ID = 6
LATIN_LANG_ID = 40
LOGLAN_LANG_ID = 58
LOJBAN_LANG_ID = 1

def format_xref(xref):
    if not xref or xref == "-":
        return ""
    parts = [p.strip() for p in xref.split(",") if p.strip()]
    formatted = ", ".join(f"{{{p}}}" for p in parts)
    return f"See also: {formatted}."

def main():
    parser = argparse.ArgumentParser(description="Import Gua\\spi dictionary via API.")
    parser.add_argument("--api-url", default=os.environ.get("LENSISKU_API_URL", API_URL_DEFAULT), help="Base API URL")
    parser.add_argument("--username", default=os.environ.get("LENSISKU_USERNAME", "gleki"), help="Username for authentication")
    parser.add_argument("--password", default=os.environ.get("LENSISKU_PASSWORD"), help="Password for authentication")
    args = parser.parse_args()

    if not args.username or not args.password:
        print("Error: Provide --username and --password or set LENSISKU_USERNAME and LENSISKU_PASSWORD env vars.", file=sys.stderr)
        return 1

    if not os.path.exists(TSV_FILE):
        print(f"Error: {TSV_FILE} not found.", file=sys.stderr)
        return 1

    # Login and get token
    login_url = f"{args.api_url}/auth/login"
    login_data = json.dumps({"username": args.username, "password": args.password}).encode("utf-8")
    req = urllib.request.Request(login_url, data=login_data, headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req) as response:
            res_body = json.loads(response.read())
            token = res_body.get("token")
            if not token:
                print("Error: No token returned after login.", file=sys.stderr)
                return 1
    except urllib.error.URLError as e:
        print(f"Login failed: {e}", file=sys.stderr)
        if hasattr(e, 'read'):
            print(e.read().decode("utf-8", errors="ignore"))
        return 1

    print(f"Successfully logged in as '{args.username}'.")

    post_url = f"{args.api_url}/jbovlaste/valsi"

    with open(TSV_FILE, "r", encoding="utf-8") as f:
        f.seek(0)
        lines = f.readlines()
        
        headers = lines[0].strip().split("\t")
        
        rows = csv.DictReader(lines, delimiter="\t")
        next(rows) # Skip the second header row

        count = 0
        for row in rows:
            word_val = row.get("!Word")
            if not word_val or word_val in ("000", "010", "011"):
                continue
            
            cls = row.get("Class")
            if cls == "t":
                continue # Skip structural headers

            selmaho = None
            jargon = None
            
            if cls in ("S", "P"):
                selmaho = cls
            elif cls in ("r", "q", "p"):
                jargon = cls

            definition_text = row.get("definition (*=obj cpd, +n=1st case merge, -n=2nd case merge, Sn=set members, Pn=pairwise, @=merge but no cpd, ?=special merge, = =as needed)") or ""
            comments = row.get("Comments (c=ch,q=sh,x=zh,#=schwa,i=ee,y=bin,w=ng)") or ""
            xref = row.get("Xref")
            
            notes = comments
            xref_formatted = format_xref(xref)
            if xref_formatted:
                notes = f"{notes} {xref_formatted}".strip()

            # Etymology
            etys = []
            for lang_col in ["Engl", "Chinese", "Latin", "Loglan"]:
                val = row.get(lang_col)
                if val and val != "-":
                    etys.append(f"{lang_col}: {val}")
            etymology = "; ".join(etys)

            # Metadata
            metadata = {k: v for k, v in row.items() if v and v != "-"}

            gloss_keywords = []
            
            glossword_map = {
                "Engl": ENGLISH_LANG_ID,
                "Chinese": CHINESE_LANG_ID,
                "Latin": LATIN_LANG_ID,
                "Loglan": LOGLAN_LANG_ID,
                "Lojban": LOJBAN_LANG_ID
            }

            for col, lang_id in glossword_map.items():
                val = row.get(col)
                if val and val != "-" and val != "":
                    keywords = re.split(r'[,/]', val)
                    for kw in keywords:
                        kw = kw.strip()
                        if not kw:
                            continue
                        gloss_keywords.append({
                            "word": kw,
                            "lang_id": lang_id,
                            "place": 0
                        })

            payload = {
                "word": word_val,
                "definition": definition_text,
                "lang_id": ENGLISH_LANG_ID,
                "source_langid": GUASPI_LANG_ID,
                "notes": notes if notes else None,
                "selmaho": selmaho,
                "jargon": jargon,
                "metadata": metadata,
                "etymology": etymology if etymology else None,
                "gloss_keywords": gloss_keywords
            }

            req_headers = {
                "Content-Type": "application/json",
                "Authorization": f"Bearer {token}"
            }
            req = urllib.request.Request(post_url, data=json.dumps(payload).encode("utf-8"), headers=req_headers, method="POST")
            
            try:
                with urllib.request.urlopen(req) as response:
                    res = json.loads(response.read())
                    count += 1
                    if count % 100 == 0:
                        print(f"Imported {count} words...")
            except urllib.error.URLError as e:
                print(f"Failed to import word '{word_val}': {e}", file=sys.stderr)
                if hasattr(e, 'read'):
                    print(e.read().decode("utf-8", errors="ignore"), file=sys.stderr)
                time.sleep(1) # Backoff a bit on error

    print(f"Successfully processed {count} Gua\\spi words.")
    return 0

if __name__ == "__main__":
    sys.exit(main())
