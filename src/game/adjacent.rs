pub fn players_are_adjacent(i: usize, j: usize, n: usize) -> bool {
    let diff = (n + i - j) % n;
    diff == 1 || diff == n - 1
}
