"""Download a public dataset linked in verify-people-work.md."""

import argparse
import shutil
from pathlib import Path
from time import sleep
from urllib.error import HTTPError
from urllib.parse import urlsplit
from urllib.request import Request, urlopen
from urllib.robotparser import RobotFileParser


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("url", help="Direct dataset URL from the verification report")
    parser.add_argument("--sample", action="store_true", help="Request the first MiB")
    args = parser.parse_args()

    parts = urlsplit(args.url)
    filename = Path(parts.path).name
    if parts.scheme != "https" or not filename or filename in (".", ".."):
        parser.error("Use a direct HTTPS file URL from the report.")

    agent = "SantiagoDataCatalog/1.0"
    headers = {"User-Agent": agent}
    robots_url = f"https://{parts.netloc}/robots.txt"
    robots = RobotFileParser(robots_url)
    try:
        with urlopen(Request(robots_url, headers=headers), timeout=30) as response:
            robots.parse(response.read().decode("utf-8", errors="replace").splitlines())
    except HTTPError as error:
        if error.code not in (403, 404, 410):
            raise
    else:
        if not robots.can_fetch(agent, args.url):
            parser.error("The publisher's robots.txt excludes this URL.")

    sleep(max(1, robots.crawl_delay(agent) or 0))
    if args.sample:
        headers["Range"] = "bytes=0-1048575"
        filename += ".sample"

    folder = Path(__file__).resolve().parents[2] / "data/raw/verify-people-work"
    folder.mkdir(parents=True, exist_ok=True)
    path = folder / filename
    with urlopen(Request(args.url, headers=headers), timeout=60) as response:
        if args.sample and response.status != 206:
            parser.error("The publisher did not return the requested byte range.")
        with path.open("wb") as output:
            shutil.copyfileobj(response, output)
        print(f"HTTP {response.status}, {path.stat().st_size:,} bytes, {path}")


if __name__ == "__main__":
    main()
