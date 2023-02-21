## deepwell-importer

A system to import Wikicomma data for eventual use in a DEEPWELL database. This outputs a SQLite database file with a minimized, more Wikidot-like schema which a DEEPWELL instance can then read and ingest.

```
$ pip3 install -r requirements.txt
$ python3 -m ingester -d [wikicomma-dir] -o [sqlite-file]
```
