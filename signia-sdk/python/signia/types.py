
from dataclasses import dataclass
from typing import List

@dataclass
class SchemaV1:
    kind: str
    nodes: list
    edges: list

@dataclass
class ManifestV1:
    schema_hash: str
    artifacts: List[str]

@dataclass
class ProofV1:
    root: str
    leaves: List[str]
