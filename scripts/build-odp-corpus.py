#!/usr/bin/env python3
"""Build deterministic public ODP qualification derivatives."""
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZIP_STORED, ZipFile, ZipInfo

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "test-corpus" / "odp" / "public-presentation.odp"
OUT = SOURCE.parent
MIMETYPE = b"application/vnd.oasis.opendocument.presentation"
FIXED_TIMESTAMP = (1980, 1, 1, 0, 0, 0)


def read_entries():
    with ZipFile(SOURCE) as archive:
        return [(info.filename, archive.read(info)) for info in archive.infolist()]


def write_entry(archive, filename, data, compression):
    info = ZipInfo(filename, date_time=FIXED_TIMESTAMP)
    info.compress_type = compression
    info.create_system = 3
    info.external_attr = 0o600 << 16
    archive.writestr(info, data)


def write_package(name, replacements=None, removed=(), extra=()):
    replacements = replacements or {}
    removed = set(removed)
    entries = []
    seen = set()
    for filename, data in read_entries():
        if filename in removed or filename in seen:
            continue
        seen.add(filename)
        entries.append((filename, replacements.get(filename, data)))
    entries.extend(extra)
    with ZipFile(OUT / name, "w") as archive:
        for filename, data in entries:
            compression = ZIP_STORED if filename == "mimetype" else ZIP_DEFLATED
            write_entry(archive, filename, data, compression)


def main():
    with ZipFile(SOURCE) as archive:
        content = archive.read("content.xml")
        manifest = archive.read("META-INF/manifest.xml")
    write_package(
        "active-content.odp",
        extra=[("Basic/Standard/Module1.xml", b"<script>disabled</script>")],
    )
    write_package(
        "external-reference.odp",
        replacements={
            "content.xml": content.replace(
                b"</office:presentation>",
                b'<draw:a xlink:href="https://example.invalid/remote">remote</draw:a></office:presentation>',
                1,
            )
        },
    )
    write_package(
        "hidden-content.odp",
        replacements={
            "content.xml": content.replace(
                b'<draw:page draw:name="Data"',
                b'<draw:page presentation:visibility="hidden" draw:name="Data"',
                1,
            )
        },
    )
    write_package(
        "missing-asset.odp",
        removed=("Pictures/TablePreview1.svm",),
    )
    write_package(
        "malformed-content.odp",
        replacements={
            "content.xml": content.replace(
                b"</office:presentation>",
                b"</office:presentatio>",
                1,
            )
        },
    )
    write_package(
        "wrong-mimetype.odp",
        replacements={"mimetype": b"application/octet-stream"},
    )
    encrypted = manifest.replace(
        b"</manifest:manifest>",
        b'<manifest:file-entry manifest:full-path="content.xml"><manifest:encryption-data/></manifest:file-entry></manifest:manifest>',
        1,
    )
    write_package(
        "encrypted.odp",
        replacements={"META-INF/manifest.xml": encrypted},
    )
    with ZipFile(OUT / "archive-amplification.odp", "w") as archive:
        write_entry(archive, "mimetype", MIMETYPE, ZIP_STORED)
        write_entry(archive, "content.xml", b"x" * (17 * 1024 * 1024), ZIP_DEFLATED)


if __name__ == "__main__":
    main()
