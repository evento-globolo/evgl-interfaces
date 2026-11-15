"""Regressions for schema-key context and line-anchored merge marker detection."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))

from validate_schemas import structural_check
from verify_repo import has_conflict_markers


class SchemaStructuralValidationTests(unittest.TestCase):
    def test_property_names_are_not_keywords(self):
        document = {
            "type": "object",
            "properties": {
                "type": {"$ref": "#/$defs/scalarType"},
                "required": {"type": "boolean"},
            },
            "$defs": {"scalarType": {"anyOf": [{"type": "string"}]}},
            "required": ["type"],
        }
        problems = []
        structural_check(document, "example.json", problems)
        self.assertEqual([], [p.message for p in problems])

    def test_invalid_schema_keyword_is_still_reported(self):
        document = {"type": "object", "properties": {"value": {"type": {"bad": True}}}}
        problems = []
        structural_check(document, "example.json", problems)
        self.assertIn("'type' must be a string or array", [p.message for p in problems])

    def test_invalid_required_keyword_is_reported(self):
        problems = []
        structural_check({"type": "object", "required": {"oops": "not a list"}}, "x", problems)
        self.assertIn("'required' must be a list", [p.message for p in problems])


class ConflictMarkersTests(unittest.TestCase):
    def test_whole_line_conflict_markers(self):
        self.assertTrue(has_conflict_markers("<<<<<<< HEAD\ncode\n=======\ncode\n>>>>>>> main\n"))

    def test_grep_expression_is_not_a_conflict(self):
        self.assertFalse(has_conflict_markers("git grep -E '^(<<<<<<< .+|=======|>>>>>>> .+)$'\n"))
        self.assertFalse(has_conflict_markers("x ======= y\n"))


if __name__ == "__main__":
    unittest.main()
