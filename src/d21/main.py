import numpy as np
from scipy.sparse import csr_matrix
from scipy.sparse.linalg import eigs

def parse_grid_to_adj_matrix(grid):
    n = len(grid)
    m = len(grid[0])
    adj_matrix = np.zeros((n*m, n*m))

    for i in range(n):
        for j in range(m):
            if grid[i][j] != "#":
                if i > 0 and grid[i-1][j] != "#":
                    adj_matrix[i*m+j, (i-1)*m+j] = 1
                if i < n-1 and grid[i+1][j] != "#":
                    adj_matrix[i*m+j, (i+1)*m+j] = 1
                if j > 0 and grid[i][j-1] != "#":
                    adj_matrix[i*m+j, i*m+j-1] = 1
                if j < m-1 and grid[i][j+1] != "#":
                    adj_matrix[i*m+j, i*m+j+1] = 1

    return adj_matrix




def count_possible_locations(adj_matrix, steps, initial_vector):
    print(adj_matrix)
    print(adj_matrix.shape)

    # Eigen decomposition
    print('Computing eigenvalues')
    eigenvalues, eigenvectors = eigs(adj_matrix, k=adj_matrix.shape[0])  # Compute all eigenvalues
    print('Done')
    D = np.diag(eigenvalues)
    V = eigenvectors
    print('Inverting V')
    V_inv = np.linalg.inv(V)
    print('Done')

    # Raising D to the power of steps
    print('Computing D_power')
    D_power = np.diag(eigenvalues**steps)

    # Recombining to get the final matrix
    print('computing matrix power')
    final_matrix = V @ D_power @ V_inv

    # Multiply with the initial vector
    print('multiplying to get final_vector')
    final_vector = final_matrix @ initial_vector
    num_possible_locations = np.sum(final_vector >= 1)

    return final_vector, num_possible_locations

def parse_vector_to_grid(vector):
    grid = []
    for i in range(n):
        row = []
        for j in range(m):
            index = i * m + j
            if grid[i][j] == "#":
                row.append("#")
            elif vector[index] <= 0.005:
                row.append(".")
            else:
                row.append("O")
        grid.append(row)
    return grid

def parse_file(filename):
    with open(filename, "r") as file:
        grid = [list(line.strip()) for line in file.readlines() if line.strip()]
        m,n = len(grid), len(grid[0])
        print("grid shape", m, n, m * n)

    return grid

# config
filename = "/Users/evanthomas/github.com/ethomas2/advent-of-code-2023/src/d21/input"
steps = 2

# parse file
grid = parse_file(filename)
nrows, ncols = len(grid), len(grid[0])
start_idx = next(nrows * r + c for r in range(nrows) for c in range(ncols) if grid[r][c] == 'S')
start_position = np.zeros(((nrows * ncols), 1))
start_position[start_idx] = 1

# compute
adj_matrix = parse_grid_to_adj_matrix(grid)
final_vector, num_possible_locations = count_possible_locations(adj_matrix, steps, start_position)
print('Answer', num_possible_locations)
# parse_vector_to_grid(final_vector), num_possible_locations
