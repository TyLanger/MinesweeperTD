use bevy::prelude::*;

// use crate::grid::{self, Coords};

pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(test_path);
    }
}

// got from
// https://www.youtube.com/watch?v=1bO1FdEThnU&ab_channel=CodeMonkey
// converted to rust
// Nodes
// have G, H, F
// G: cost from the start node
// H: Heurostoc cost to reach end
// F: G + H
// open list - need to evaluate
// closed list - already evaluated

const STRAIGHT_COST: usize = 10;
const DIAGONAL_COST: usize = 14;

#[derive(Copy, Clone)]
struct PathNode {
    x: usize,
    y: usize,

    index: usize,

    g: usize,
    h: usize,
    f: usize,

    is_walkable: bool,

    came_from_index: Option<usize>,
}

impl PathNode {
    fn calculate_f_cost(&mut self) {
        if self.g > usize::MAX - self.h {
            // too big
            // don't add to max
            self.f = usize::MAX;
        } else {
            self.f = self.g + self.h;
        }
    }
}

fn test_path() {
    find_path(0, 0, 3, 1);
    // Path is:
    // (1, 3)
    // (2, 1)
    // (1, 1)
    // (0, 0)
    // should be
    // (3, 1)
    // (2, 1)
    // (1, 0)
    // (0, 0)
    // using reverse indices,
    // Path is:
    // (3, 1)
    // (2, 1)
    // (1, 1)
    // (0, 0)
    // which is correct
    // So why is his way correct for him?
    // what is backwards in mine?
    // do unity nativelists add to the front?
    // maybe because he does array[index] = item
    // which isn't the order the loops run in
    // they are created down to up, left to right
    // vec.push is down to up, left to right
    // he inserts them left to right, down to up
}

pub fn find_path(start_x: usize, start_y: usize, end_x: usize, end_y: usize) {
    let grid_width = 4;
    let grid_height = 5;

    let mut nodes = Vec::new();

    for i in 0..grid_width {
        for j in 0..grid_height {
            let g = usize::MAX;
            let h = calculate_dist_cost(i, j, end_x, end_y);
            let index = calculate_index(i, j, grid_height);
            let f = g; // g + h, but g is max value so don't want overflow
            let node = PathNode {
                x: i,
                y: j,
                index,
                g,
                h,
                f,
                is_walkable: true,
                came_from_index: None,
            };
            nodes.push(node);
        }
    }

    let index = calculate_index(start_x, start_y, grid_height);
    if let Some(mut start_node) = nodes.get_mut(index) {
        start_node.g = 0;
        start_node.calculate_f_cost();

        let mut open_list: Vec<usize> = Vec::new();
        let mut closed_list: Vec<usize> = Vec::new();

        open_list.push(start_node.index);

        let end_index = calculate_index(end_x, end_y, grid_height);

        let mut open_list_len = open_list.len().clone();
        let mut count = 0;
        while open_list_len > 0 && count < 50 {
            count += 1;
            if let Some(current_node_index) = get_lowest_fcost_index(&open_list, &nodes) {
                if current_node_index == end_index {
                    // reached dest
                    break;
                }

                let current_node = nodes.get(current_node_index);

                if let Some(current_node) = current_node {
                    let current_node = current_node.clone();
                    // remove the current node from the open list
                    for (i, item) in open_list.iter().enumerate() {
                        if *item == current_node_index {
                            open_list.remove(i);
                            open_list_len = open_list.len().clone();

                            break;
                        }
                    }

                    closed_list.push(current_node.index);

                    let neighbours = get_neighbour_indicies(
                        current_node.x,
                        current_node.y,
                        grid_width,
                        grid_height,
                    );

                    // get neighbours
                    for n in neighbours {
                        if closed_list.contains(&n) {
                            continue;
                        }

                        if let Some(neighbour_node) = nodes.get_mut(n) {
                            // check if node is walkable
                            if !neighbour_node.is_walkable {
                                continue;
                            }

                            let tentative_g_cost = current_node.g
                                + calculate_dist_cost(
                                    current_node.x,
                                    current_node.y,
                                    neighbour_node.x,
                                    neighbour_node.y,
                                );
                            if tentative_g_cost < neighbour_node.g {
                                neighbour_node.came_from_index = Some(current_node_index);
                                neighbour_node.g = tentative_g_cost;
                                neighbour_node.calculate_f_cost();

                                if !open_list.contains(&neighbour_node.index) {
                                    open_list.push(neighbour_node.index);
                                    open_list_len = open_list.len().clone();
                                }
                            }
                        }
                    }
                }
            } else {
                // couldn't find a lowest fcost in the open set
                // shouldn't be possible
            }
        }

        if let Some(end_node) = nodes.get(end_index) {
            if end_node.came_from_index.is_none() {
                // didn't find a path
                println!("Didn't find a path");
            } else {
                // did find a path
                let path = calculate_path(&nodes, end_node);
                println!("Path is: ");
                for p in path {
                    println!("({:?}, {:?})", p.x, p.y);
                }
            }
        }
    }
}

// #[derive(Debug)]
struct PathPoints {
    x: usize,
    y: usize,
}

fn calculate_path(nodes: &Vec<PathNode>, end_node: &PathNode) -> Vec<PathPoints> {
    let mut v = Vec::new();

    if let Some(_came_from) = end_node.came_from_index {
        // do I need this base case?
        // doesn't the loop handle it?
        v.push(PathPoints {
            x: end_node.x,
            y: end_node.y,
        });

        let mut current_node = end_node;
        while current_node.came_from_index.is_some() {
            if let Some(came_from_index) = current_node.came_from_index {
                if let Some(came_from_node) = nodes.get(came_from_index) {
                    v.push(PathPoints {
                        x: came_from_node.x,
                        y: came_from_node.y,
                    });
                    current_node = came_from_node;
                }
            }
        }
    }

    v
}

fn get_neighbour_indicies(x: usize, y: usize, width: usize, height: usize) -> Vec<usize> {
    let mut v = Vec::new();
    let h = height - 1;
    let w = width - 1;
    if y < h {
        v.push(calculate_index(x, y + 1, height)); // up
    }
    if y < h && x < w {
        v.push(calculate_index(x + 1, y + 1, height)); // up right
    }
    if x < w {
        v.push(calculate_index(x + 1, y, height)); // right
    }
    if y > 0 {
        if x < w {
            v.push(calculate_index(x + 1, y - 1, height)); // down right
        }
        v.push(calculate_index(x, y - 1, height)); // down
    }
    if y > 0 && x > 0 {
        v.push(calculate_index(x - 1, y - 1, height)); // down left
    }
    if x > 0 {
        v.push(calculate_index(x - 1, y, height)); // left
        if y < h {
            v.push(calculate_index(x - 1, y + 1, height)); // up left
        }
    }

    v
}

fn get_lowest_fcost_index(open_list: &Vec<usize>, nodes: &Vec<PathNode>) -> Option<usize> {
    open_list
        .iter()
        .min_by_key(|x| nodes.get(**x).unwrap().f)
        .map(|x| *x)
}

fn calculate_index(x: usize, y: usize, grid_height: usize) -> usize {
    //this gives
    // 12 13 14 15
    // 8 9 10 11
    // 4 5 6 7
    // 0 1 2 3
    // x + y * grid_width
    // this gives
    // 3 7 11 15
    // 2 6 10 14
    // 1 5 9 13
    // 0 4 8 12
    // this is the order elements are created if
    // for x { for y { }}
    y + x * grid_height
}

fn calculate_dist_cost(a_x: usize, a_y: usize, b_x: usize, b_y: usize) -> usize {
    // let x_dist = if a_x > b_x { a_x - b_x } else { b_x - a_x };
    // let y_dist = if a_y > b_y { a_y - b_y } else { b_y - a_y };
    // let remainder = if x_dist > y_dist {
    //     x_dist - y_dist
    // } else {
    //     y_dist - x_dist
    // };

    // (0,1) -> (3,1)
    // 3, 0
    // r = 3
    // 0*14 + 3*10 = 30

    let x_dist = a_x.abs_diff(b_x);
    let y_dist = a_y.abs_diff(b_y);
    let remainder = x_dist.abs_diff(y_dist);
    DIAGONAL_COST * x_dist.min(y_dist) + STRAIGHT_COST * remainder
}
