#!/usr/bin/env python3

import argparse
import logging
import sys
from . import run_wikicomma_import

LOG_FORMAT = "[%(levelname)s] %(asctime)s %(name)s: %(message)s"
LOG_DATE_FORMAT = "[%Y/%m/%d %H:%M:%S]"

if __name__ == "__main__":
    argparser = argparse.ArgumentParser(description="WikiComma importer")
    argparser.add_argument(
        "-q",
        "--quiet",
        "--no-stdout",
        dest="stdout",
        action="store_false",
        help="Don't output to standard out",
    )
    argparser.add_argument(
        "-D",
        "--debug",
        dest="debug",
        action="store_true",
        help="Set logging level to debug",
    )
    argparser.add_argument(
        "-d",
        "--directory",
        "--wikicomma-directory",
        dest="wikicomma_directory",
        required=True,
        help="The directory where WikiComma data resides",
    )
    argparser.add_argument(
        "-o",
        "--sql",
        "--output-sql",
        dest="sqlite_path",
        required=True,
        help="The path of the SQLite database to write to",
    )
    argparser.add_argument(
        "-k",
        "--colon",
        "--replace-colons",
        dest="replace_colons",
        action="store_true",
        help="Whether files use underscores instead of colons in filenames",
    )
    args = argparser.parse_args()

    log_fmtr = logging.Formatter(LOG_FORMAT, datefmt=LOG_DATE_FORMAT)
    log_stdout = logging.StreamHandler(sys.stdout)
    log_stdout.setFormatter(log_fmtr)
    log_level = logging.DEBUG if args.debug else logging.INFO

    logger = logging.getLogger(__name__)
    logger.setLevel(level=log_level)
    logger.addHandler(log_stdout)

    ingester = Ingester(
        args.wikicomma_directory, args.sqlite_database_path, args.replace_colons,
    )
    try:
        ingester.setup()
        ingester.ingest_users()
        ingester.ingest_sites()
    finally:
        ingester.close()
