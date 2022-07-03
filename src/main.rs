extern crate minifb;

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use rand::Rng;

const WIDTH: usize = 160;
const HEIGHT: usize = 160;

const WHITE: u32 = 0xFFFFFF;
const BLACK: u32 = 0x000000;
const START: u32 = 0x0000FF;
const END  : u32 = 0xFF0000;

const EXPAND : u32 = 0x00FF00;
const SOLUTION_START: u32 = 0x0088FF;
const SOLUTION_FINISH: u32 = 0x8888FF;
const SELF_DESTRUCT: u32 = 0xFF00FF;

const QUICK_MODE: bool = false;
const DIAGONALS: bool = false;
const RANDOM_START_AND_END: bool = false;

const DIAG_DIRECTIONS: [Direction; 9] = [
    Direction::Topleft,
    Direction::Top,
    Direction::Topright,
    Direction::Left,
    Direction::Center,
    Direction::Right,
    Direction::Bottomleft,
    Direction::Bottom,
    Direction::Bottomright,
];

const DIRECTIONS: [Direction; 5] = [
    Direction::Top,
    Direction::Left,
    Direction::Center,
    Direction::Right,
    Direction::Bottom,
];

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let directions: Vec<Direction> = if DIAGONALS {
        DIAG_DIRECTIONS.iter().map(|d| *d).collect::<Vec<Direction>>()
    } else {
        DIRECTIONS.iter().map(|d| *d).collect::<Vec<Direction>>()
    };

    let mut window = Window::new(
        "Blob Pathfinding",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            scale: Scale::X4,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    buffer = sidewinder();
    buffer[2 * WIDTH - 2] = EXPAND;

    let mut buffer: Vec<Tile> = buffer.iter().map(|c| Tile{color: *c, parent_direction: Direction::Center}).collect::<Vec<Tile>>();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for blob in buffer.clone().iter().enumerate().filter(|t| ![WHITE, BLACK, END].contains(&t.1.color)) {
            let coordinate = blob.0;
            
            if blob.1.color == EXPAND {
                for dir in directions.clone() {
                    let target_coordinate = find_target_index(coordinate, dir);
                    let target = buffer[target_coordinate];
                    if target.color == WHITE {
                        buffer[target_coordinate] = Tile{
                            color: EXPAND,
                            parent_direction: reverse_direction(dir),
                        }
                    }

                    if target.color == END {
                        buffer[coordinate].color = SOLUTION_START;
                        buffer[find_target_index(coordinate, blob.1.parent_direction)].color = SOLUTION_START;
                    }

                    if target.color == SOLUTION_FINISH || target.color == SELF_DESTRUCT {
                        buffer[coordinate].color = SELF_DESTRUCT;
                    }
                }
            }

            if blob.1.color == SOLUTION_START {
                buffer[coordinate].color = SOLUTION_START;
                let parent_index = find_target_index(coordinate, blob.1.parent_direction);
                if buffer[parent_index].color == EXPAND {
                    buffer[parent_index].color = SOLUTION_START
                };

                if !QUICK_MODE {
                    for dir in directions.clone() {
                        let target_index = find_target_index(coordinate, dir);
                        if buffer[target_index].color == START || buffer[target_index].color == SOLUTION_FINISH {
                            buffer[coordinate].color = SOLUTION_FINISH;
                        }
                    }
                }
            }

            if blob.1.color == SELF_DESTRUCT {
                buffer[coordinate].color = WHITE;
            }
        }
        
        window
            .update_with_buffer(&(buffer.iter().map(|t| t.color).collect::<Vec<u32>>()), WIDTH, HEIGHT)
            .unwrap();
    }
}

fn sidewinder() -> Vec<u32> {
    let mut maze: Vec<u32> = vec![BLACK; WIDTH*HEIGHT];
    let mut rng = rand::thread_rng();

    let mut run: Vec<Point> = Vec::new();

    for y in (0..HEIGHT-1).rev() {
        for x in 0..WIDTH-1 {
            if (y+1) % 2 == 0 && (x+1) % 2 == 0 {
                set_maze_point(&mut maze, Point{x, y});
                
                let far_east: bool = x == WIDTH - 3;
                let far_north: bool = y == 1;

                if far_east && far_north {
                    run = Vec::new();
                    continue;
                }

                run.push(Point{x, y});

                if far_east {
                    // Clears north wall.
                    let chosen_point = run[rng.gen_range(0..run.len())];
                    set_maze_point(&mut maze, Point{x: chosen_point.x, y: chosen_point.y-1});
                    run = Vec::new();
                    continue;
                }

                if far_north {
                    // Clears East wall.
                    set_maze_point(&mut maze, Point{x: x+1, y});
                    continue;
                }

                if rng.gen() {
                    set_maze_point(&mut maze, Point{x: x+1, y});
                } else {
                    let chosen_point = run[rng.gen_range(0..run.len())];
                    set_maze_point(&mut maze, Point{x: chosen_point.x, y: chosen_point.y-1});
                    run = Vec::new();
                }
            }
        }
    }

    if RANDOM_START_AND_END {
    } else {
        maze[f2d_to_1d(Point{x: 1, y: HEIGHT-2})] = END;
        maze[2 * WIDTH - 1] = START;
    }

    maze
}

fn find_target_index(index: usize, direction: Direction) -> usize {
    match direction {
        Direction::Topleft => {
            index - WIDTH - 1
        },
        Direction::Top => {
            index - WIDTH
        },
        Direction::Topright => {
            index - WIDTH + 1
        },
        Direction::Left => {
            index - 1
        },
        Direction::Center => {
            index
        },
        Direction::Right => {
            index + 1
        },
        Direction::Bottomleft => {
            index + WIDTH - 1
        },
        Direction::Bottom => {
            index + WIDTH
        },
        Direction::Bottomright => {
            index + WIDTH + 1
        }
    }
}

fn reverse_direction(direction: Direction) -> Direction {
    match direction {
        Direction::Topleft => Direction::Bottomright,
        Direction::Top => Direction::Bottom,
        Direction::Topright => Direction::Bottomleft,
        Direction::Left => Direction::Right,
        Direction::Center => Direction::Center,
        Direction::Right => Direction::Left,
        Direction::Bottomleft => Direction::Topright,
        Direction::Bottom => Direction::Top,
        Direction::Bottomright => Direction::Topleft,
    }
}

fn set_maze_point(maze: &mut Vec<u32>, point: Point) {
    maze[f2d_to_1d(point)] = WHITE;
}

fn f2d_to_1d(p: Point) -> usize {
    (p.y * (WIDTH) + p.x) as usize
}

#[derive(Copy, Clone, Debug)]
enum Direction {
    Topleft,
    Top,
    Topright,
    Left,
    Center,
    Right,
    Bottomleft,
    Bottom,
    Bottomright
}

#[derive(Copy, Clone, Debug)]
struct Tile {
    color: u32,
    parent_direction: Direction,
}

#[derive(Copy, Clone, Debug)]
struct Point {
    x: usize,
    y: usize
}