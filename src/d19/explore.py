"""
Script to prove that the workflow graph for d19 is a tree. I actually don't
think it's necessary to know if it's a tree or not, but I was curious. Turns
out it's a tree
"""


def parse_line(line):
    name = line.split('{')[0]
    idx = next(i for (i, ch) in enumerate(line) if ch == '{')
    parts = line[idx + 1: -1].split(',')
    destinations = [p.split(':')[-1] for p in parts[:-1]] + [parts[-1]]
    return (name, destinations)


def is_tree(start, workflow_graph, visited = None):
    if start in ['A', 'R']:
        return True
    visited = visited or set()

    if start in visited:
        print(f'found loop involving {start}')
        return False

    child_nodes = workflow_graph[start]
    return all(
        is_tree(cn, workflow_graph, visited | set([start]))
        for cn in child_nodes
    )





if __name__ == '__main__':
    with open('src/d19/input') as f:
        lines = [line.strip() for line in f.readlines() ]
        empty_line = next(i for (i, line) in enumerate(lines) if line == '')
        lines = lines[:empty_line]

    parsed_lines = [ parse_line(line) for line in lines ]

    workflow_graph = {src: destinations for (src, destinations) in parsed_lines}
    print(is_tree('in', workflow_graph))
