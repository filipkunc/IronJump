#!/usr/bin/env python3
"""Convert IronJump NSKeyedArchiver .level files to the editor's .xlevel format.

Run from any directory:
    python3 tools/convert_levels.py
    python3 tools/convert_levels.py --check

Defaults to IronJump/Levels -> IronJump/XLevels/Converted. Only Python's standard
library is required; archived Objective-C classes are never instantiated.
"""

import argparse
import math
import plistlib
from pathlib import Path
import xml.etree.ElementTree as ET


REPO_ROOT = Path(__file__).resolve().parents[1]

# Field order follows each class's writeToXML: method. Optional defaults mirror
# initWithCoder: for older archives, rather than relying on XML class defaults.
SCHEMAS = {
    "FPPlayer": ("x", "y"),
    "FPPlatform": ("x", "y", "widthSegments", "heightSegments"),
    "FPMovablePlatform": ("x", "y", "widthSegments", "heightSegments"),
    "FPElevator": ("x", "y", "endX", "endY", "widthSegments"),
    "FPDiamond": ("x", "y"),
    "FPExit": ("x", "y"),
    "FPTrampoline": ("x", "y", "widthSegments"),
    "FPMagnet": ("x", "y", "widthSegments"),
    "FPSpeedPowerUp": ("x", "y"),
}
DEFAULTS = {
    ("FPPlatform", "heightSegments"): 1,
    ("FPMovablePlatform", "heightSegments"): 1,
    ("FPTrampoline", "widthSegments"): 1,
    ("FPMagnet", "widthSegments"): 1,
}
INTEGER_FIELDS = {"widthSegments", "heightSegments"}


def decode_level(data: bytes) -> list[tuple[str, dict[str, int | float]]]:
    archive = plistlib.loads(data)
    if not isinstance(archive, dict) or archive.get("$archiver") != "NSKeyedArchiver":
        raise ValueError("Expected an NSKeyedArchiver property list")
    objects = archive.get("$objects")
    if not isinstance(objects, list):
        raise ValueError("Missing archive object table")

    def dereference(reference):
        if not isinstance(reference, plistlib.UID) or reference.data >= len(objects):
            raise ValueError(f"Invalid archive reference: {reference!r}")
        return objects[reference.data]

    def class_name(record):
        if not isinstance(record, dict):
            raise ValueError("Expected an archived object dictionary")
        metadata = dereference(record.get("$class"))
        if not isinstance(metadata, dict) or "$classname" not in metadata:
            raise ValueError("Missing archived class name")
        return metadata["$classname"]

    root = dereference(archive.get("$top", {}).get("root"))
    if class_name(root) not in {"NSArray", "NSMutableArray"}:
        raise ValueError("Expected an array of game objects at the archive root")
    references = root.get("NS.objects")
    if not isinstance(references, list):
        raise ValueError("Missing root array contents")

    result = []
    for reference in references:
        record = dereference(reference)
        name = class_name(record)
        if name not in SCHEMAS:
            raise ValueError(f"Unsupported game object: {name}")
        unexpected = set(record) - set(SCHEMAS[name]) - {"$class"}
        if unexpected:
            raise ValueError(f"Unsupported fields in {name}: {sorted(unexpected)}")

        fields = {}
        for field in SCHEMAS[name]:
            if field in record:
                value = record[field]
            elif (name, field) in DEFAULTS:
                value = DEFAULTS[name, field]
            else:
                raise ValueError(f"Missing required field: {name}.{field}")
            if field in INTEGER_FIELDS:
                if type(value) is not int:
                    raise ValueError(f"Expected integer: {name}.{field}")
            elif type(value) not in (int, float) or not math.isfinite(value):
                raise ValueError(f"Expected finite number: {name}.{field}")
            fields[field] = value
        result.append((name, fields))
    return result


def encode_xlevel(objects: list[tuple[str, dict[str, int | float]]]) -> bytes:
    root = ET.Element("IronJumpLevel")
    for name, fields in objects:
        node = ET.SubElement(root, name)
        for field, value in fields.items():
            if field in INTEGER_FIELDS:
                text = str(value)
            else:
                # Match the editor's six decimals when lossless; retain full
                # precision for any coordinates needing more decimal places.
                text = f"{value:.6f}"
                if float(text) != value:
                    text = repr(value)
            ET.SubElement(node, field).text = text
    ET.indent(root, space="  ")
    data = ET.tostring(root, encoding="utf-8", xml_declaration=True) + b"\n"
    verify_xlevel(data, objects)
    return data


def verify_xlevel(data: bytes, expected: list[tuple[str, dict[str, int | float]]]):
    root = ET.fromstring(data)
    if root.tag != "IronJumpLevel" or root.attrib or len(root) != len(expected):
        raise ValueError("XML root or object count does not match the archive")
    for node, (name, fields) in zip(root, expected):
        if node.tag != name or node.attrib or [child.tag for child in node] != list(fields):
            raise ValueError(f"XML object order or fields do not match: {name}")
        for child in node:
            if child.attrib or len(child):
                raise ValueError(f"Unexpected XML content: {name}.{child.tag}")
            parse = int if child.tag in INTEGER_FIELDS else float
            if parse(child.text) != fields[child.tag]:
                raise ValueError(f"XML value differs from archive: {name}.{child.tag}")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("source", nargs="?", type=Path, default=REPO_ROOT / "IronJump/Levels",
                        help="A .level file or directory (default: IronJump/Levels)")
    parser.add_argument("--output-dir", type=Path, default=REPO_ROOT / "IronJump/XLevels/Converted")
    parser.add_argument("--check", action="store_true", help="Verify existing XML against each archive without writing")
    parser.add_argument("--overwrite", action="store_true", help="Replace existing outputs with different contents")
    args = parser.parse_args()
    sources = sorted(args.source.glob("*.level")) if args.source.is_dir() else [args.source]
    if not sources:
        parser.error(f"No .level files found in {args.source}")

    # Decode and verify the entire batch before writing any output.
    outputs = []
    count = 0
    for source in sources:
        try:
            objects = decode_level(source.read_bytes())
            output = args.output_dir / (source.stem + ".xlevel")
            data = encode_xlevel(objects)
            if args.check:
                verify_xlevel(output.read_bytes(), objects)
            elif output.exists() and output.read_bytes() != data and not args.overwrite:
                raise ValueError(f"Output already exists with different contents: {output}; use --overwrite")
            outputs.append((output, data))
            count += len(objects)
        except (OSError, ValueError, TypeError, KeyError, ET.ParseError, plistlib.InvalidFileException) as error:
            parser.error(f"{source}: {error}")

    if not args.check:
        args.output_dir.mkdir(parents=True, exist_ok=True)
        for output, data in outputs:
            output.write_bytes(data)
    action = "Verified" if args.check else "Converted and verified"
    print(f"{action} {len(outputs)} levels, {count} objects: {args.output_dir}")


if __name__ == "__main__":
    main()
