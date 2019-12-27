use grid::{decompose, Direction, Pos, DIRECTIONS};
use minheap::MinHeap;
use std::collections::HashMap;

//  # # # # # # # # # #
// # . # 5 . # 6 . . . #
//  @ - X - - X # . . #
// # . \ \ \ \ \ # . . #
//  # # X # \ \ \ # . #
// # . 1 \ # \ \ \ # # #
//  # . . \ # # \ X 4 #
// # . . . # # # X # # #
//  # . . . . . 2 X 3 #
// # . . . . . . . \ . #
//  # # # # # # # # # #

trait JPSearchable {
    fn passable(&self, pos: Pos) -> bool;
    fn is_goal(&self, pos: Pos) -> bool;
    fn heuristic(&self, pos: Pos) -> u32;
}

/// Returns the path as a stack. If you treat it as a list, it will
/// be backwards.
pub(super) fn jps<FG, FP, FH>(
    origin: Pos,
    is_goal: FG,
    passable: FP,
    heuristic: FH,
    flip: bool,
) -> Option<Vec<Pos>>
where
    FG: Fn(Pos) -> bool,
    FP: Fn(Pos) -> bool,
    FH: Fn(Pos) -> u32,
{
    if is_goal(origin) {
        return Some(vec![origin]);
    }
    let mut open = MinHeap::new();
    let mut costs: HashMap<Pos, u32> = HashMap::new();
    let mut parents: HashMap<Pos, JumpPoint> = HashMap::new();
    let initial_priority = heuristic(origin);
    for &direction in &DIRECTIONS {
        open.push(OpenNode::initial(origin, direction, flip), initial_priority);
    }
    costs.insert(origin, 0);
    while let Some(node) = open.pop() {
        match node {
            OpenNode::Goal(pos) => {
                return Some(construct_path(&parents, pos, costs[&pos]));
            }
            OpenNode::JumpPoint(curr) => {
                curr.clone().for_each_neighbor(
                    |neighbor| {
                        let neighbor_pos = neighbor.pos();
                        let new_cost = costs[&curr.pos] + neighbor.pos().distance(curr.pos);
                        if let Some(&cost) = costs.get(&neighbor_pos) {
                            // normally we would skip a neighbor if its cost was equal to the cost found already
                            // here we don't because in this implementation of jps,
                            // multiple neighbors can be created for a single position
                            if new_cost > cost {
                                return;
                            }
                        }
                        open.push(neighbor, new_cost + heuristic(neighbor_pos));
                        parents.insert(neighbor_pos, curr.clone());
                        costs.insert(neighbor_pos, new_cost);
                    },
                    &is_goal,
                    &passable,
                );
            }
        }
    }
    None
}

#[derive(Eq, PartialEq)]
enum OpenNode {
    Goal(Pos),
    JumpPoint(JumpPoint),
}

#[derive(Eq, PartialEq, Clone)]
struct JumpPoint {
    pos: Pos,
    direction: Direction,
    chirality: Chirality,
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum Chirality {
    Clockwise,
    Counterclockwise,
}

impl OpenNode {
    fn initial(pos: Pos, direction: Direction, flip: bool) -> Self {
        OpenNode::JumpPoint(JumpPoint {
            pos,
            direction,
            chirality: if flip {
                Chirality::Counterclockwise
            } else {
                Chirality::Clockwise
            },
        })
    }

    fn pos(&self) -> Pos {
        match *self {
            OpenNode::Goal(pos) => pos,
            OpenNode::JumpPoint(JumpPoint { pos, .. }) => pos,
        }
    }
}

struct Row {
    jump_point: JumpPoint,
    index: u32, // distance from jump point stem
    start: u32, // distance along row (inclusive) 
    end: u32, // distance along row (inclusive)
}

impl<'a> Row {
    fn search<FC, FG, FP>(&self, callback: &mut FC, is_goal: &FG, passable: &FP)
    where
        FC: FnMut(OpenNode),
        FG: Fn(Pos) -> bool,
        FP: Fn(Pos) -> bool,
    {
        let jp = &self.jump_point;
        let leaf_direction = jp.chirality.rotate(jp.direction, 1);
        let origin = jp.pos + leaf_direction * self.index;

        let prestart_pos = origin + jp.direction * (self.start - 1);
        if passable(prestart_pos) {

        }
        
    }
}

impl JumpPoint {
    fn neighbor_of(pos: Pos, old_direction: Direction, new_chirality: Chirality) -> Self {
        JumpPoint {
            pos,
            direction: new_chirality.rotate(old_direction, -1),
            chirality: new_chirality,
        }
    }

    fn for_each_neighbor2<FC, FG, FP>(&self, mut callback: FC, is_goal: &FG, passable: &FP)
    where
        FC: FnMut(OpenNode),
        FG: Fn(Pos) -> bool,
        FP: Fn(Pos) -> bool,
    {
        let mut stem_len = 0;
        loop {
            let pos = self.pos + self.direction * (stem_len + 1);
            if passable(pos) {
                stem_len += 1;
            } else {
                break;
            }
            if is_goal(pos) {
                callback(OpenNode::Goal(pos));
                break;
            }
        }
    }

    fn for_each_neighbor<FC, FG, FP>(&self, mut callback: FC, is_goal: &FG, passable: &FP)
    where
        FC: FnMut(OpenNode),
        FG: Fn(Pos) -> bool,
        FP: Fn(Pos) -> bool,
    {
        let leaf_direction = self.chirality.rotate(self.direction, 1);
        for len in 1.. {
            let pos = self.pos + self.direction * len;
            if !passable(pos) {
                break;
            }
            if is_goal(pos) {
                callback(OpenNode::Goal(pos));
                break;
            }
            let neighbor = JumpPoint::neighbor_of(pos, self.direction, self.chirality);
            if neighbor.is_forced(&passable) {
                callback(OpenNode::JumpPoint(neighbor));
            }
            for_each_leaf_neighbor(pos, leaf_direction, &mut callback, is_goal, passable);
        }
    }

    fn is_forced<FP>(&self, passable: &FP) -> bool
    where
        FP: Fn(Pos) -> bool,
    {
        let corner = self.pos + self.chirality.rotate(self.direction, -1);
        !passable(corner) && passable(self.pos + self.direction)
    }
}

fn for_each_leaf_neighbor<FC, FG, FP>(
    root: Pos,
    direction: Direction,
    callback: &mut FC,
    is_goal: &FG,
    passable: &FP,
) where
    FC: FnMut(OpenNode),
    FG: Fn(Pos) -> bool,
    FP: Fn(Pos) -> bool,
{
    for len in 1.. {
        let pos = root + direction * len;
        if !passable(pos) {
            break;
        }
        if is_goal(pos) {
            callback(OpenNode::Goal(pos));
            break;
        }
        let neighbor1 = JumpPoint::neighbor_of(pos, direction, Chirality::Clockwise);
        let neighbor2 = JumpPoint::neighbor_of(pos, direction, Chirality::Counterclockwise);
        if neighbor1.is_forced(&passable) {
            callback(OpenNode::JumpPoint(neighbor1));
        }
        if neighbor2.is_forced(&passable) {
            callback(OpenNode::JumpPoint(neighbor2));
        }
    }
}

impl Chirality {
    fn rotate(self, direction: Direction, n: i32) -> Direction {
        match self {
            Chirality::Clockwise => direction.rotate(n),
            Chirality::Counterclockwise => direction.rotate(-n),
        }
    }
}

fn construct_path(parents: &HashMap<Pos, JumpPoint>, goal: Pos, total_cost: u32) -> Vec<Pos> {
    let mut path = Vec::with_capacity(total_cost as usize);
    let mut pos = goal;
    while let Some(&JumpPoint {
        pos: parent_pos,
        direction: stem_direction,
        chirality,
    }) = parents.get(&pos)
    {
        let leaf_direction = chirality.rotate(stem_direction, 1);
        let (stem_cost, leaf_cost) = decompose(pos - parent_pos, stem_direction, leaf_direction);
        let stem_tip = parent_pos + stem_direction * stem_cost;
        for x in (1..=leaf_cost).rev() {
            path.push(stem_tip + leaf_direction * x);
        }
        for y in (1..=stem_cost).rev() {
            path.push(parent_pos + stem_direction * y);
        }
        pos = parent_pos;
    }
    path
}
