import pprint
from typing import List
from dataclasses import dataclass

@dataclass
class Module:
    type: str
    name: str
    connections: List[str]

def parse_line(line):
    a, b = line.split('->')
    a, b = a.strip(), b.strip()
    type, name = (None, None)
    match a[0]:
        case '%':
            type, name = 'flipflop', a[1:]
        case '&':
            type, name = 'conjuction', a[1:]
        case _ if a == 'broadcaster':
            type, name = 'broadcaster', 'broadcaster'
        case _:
            raise Exception(f'Unexpected first char {a[0]}')

    connections = b.split(',')
    return Module(
        type = type,
        name = name,
        connections = connections,
    )


def is_tree(start, graph, visited = None):
    """ Copy pasted from d19/explore.py """

    visited = visited or []

    if start in visited:
        print(f'found loop involving {start}', visited)
        return False

    child_nodes = graph[start]
    return all(
        is_tree(cn, graph, visited + [start])
        for cn in child_nodes
    )

with open('src/d20/input') as f:
    lines = [line.strip() for line in f.readlines() if line.strip()]

module = [parse_line(line) for line in lines]
graph = {
        mod.name: mod.connections
        for mod in module
}

print(is_tree('broadcaster', graph))
