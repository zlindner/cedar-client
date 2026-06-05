use std::collections::HashMap;
use std::collections::HashSet;

use nx_pkg4::{Node, NxNode};

const MAX_CONNECTED_FOOTHOLD_Y_DELTA: f32 = 16.0;

#[derive(Clone, Copy)]
pub struct MapRange {
    pub min: f32,
    pub max: f32,
}

pub struct MapPhysics {
    footholds: HashMap<u16, Foothold>,
    footholds_by_x: HashMap<i16, Vec<u16>>,
    walls: MapRange,
    borders: MapRange,
}

impl MapPhysics {
    pub fn from_node(map: NxNode<'_>) -> Option<Self> {
        let foothold_root = map.get("foothold")?;
        let mut footholds = HashMap::new();
        let mut footholds_by_x: HashMap<i16, Vec<u16>> = HashMap::new();
        let mut left_wall = i16::MAX;
        let mut right_wall = i16::MIN;
        let mut top_border = i16::MAX;
        let mut bottom_border = i16::MIN;

        for base in foothold_root.iter().ok()? {
            let layer = base.name().ok()?.parse::<u8>().ok()?;

            for mid in base.iter().ok()? {
                for node in mid.iter().ok()? {
                    let id = node.name().ok()?.parse::<u16>().ok()?;
                    let foothold = Foothold::from_node(node, id, layer)?;

                    left_wall = left_wall.min(foothold.left());
                    right_wall = right_wall.max(foothold.right());
                    top_border = top_border.min(foothold.top());
                    bottom_border = bottom_border.max(foothold.bottom());

                    if !foothold.is_wall() {
                        for x in foothold.left()..=foothold.right() {
                            footholds_by_x.entry(x).or_default().push(id);
                        }
                    }

                    footholds.insert(id, foothold);
                }
            }
        }

        let (walls, borders) = map_info_ranges(map).unwrap_or((
            MapRange {
                min: f32::from(left_wall + 25),
                max: f32::from(right_wall - 25),
            },
            MapRange {
                min: f32::from(top_border - 300),
                max: f32::from(bottom_border + 100),
            },
        ));

        Some(Self {
            footholds,
            footholds_by_x,
            walls,
            borders,
        })
    }

    pub fn ground_below(&self, x: f32, y: f32) -> Option<Ground> {
        let x_key = x.floor() as i16;
        let mut closest_ground = self.borders.max;
        let mut closest_foothold = None;

        for foothold_id in self.footholds_by_x.get(&x_key)? {
            let Some(foothold) = self.footholds.get(foothold_id) else {
                continue;
            };
            let ground_y = foothold.y_at(x);

            if closest_ground >= ground_y && ground_y >= y {
                closest_ground = ground_y;
                closest_foothold = Some(foothold);
            }
        }

        closest_foothold.map(|foothold| foothold.ground_at(x))
    }

    pub fn ground_for_player(
        &self,
        foothold_id: u16,
        x: f32,
        y: f32,
        grounded: bool,
    ) -> Option<Ground> {
        if grounded {
            return self.connected_ground(foothold_id, x, y);
        }

        self.ground_below(x, y)
    }

    pub fn clamp_x(&self, x: f32) -> f32 {
        x.clamp(self.walls.min, self.walls.max)
    }

    pub fn walls(&self) -> MapRange {
        self.walls
    }

    pub fn borders(&self) -> MapRange {
        self.borders
    }

    pub fn debug_foothold_lines(&self) -> Vec<MapLine> {
        self.footholds
            .values()
            .filter(|foothold| !foothold.is_wall())
            .map(|foothold| MapLine {
                id: foothold.id,
                x1: f32::from(foothold.x1),
                y1: f32::from(foothold.y1),
                x2: f32::from(foothold.x2),
                y2: f32::from(foothold.y2),
            })
            .collect()
    }

    fn connected_ground(&self, foothold_id: u16, x: f32, y: f32) -> Option<Ground> {
        let mut next_id = foothold_id;
        let mut visited = HashSet::new();

        while next_id != 0 && visited.insert(next_id) {
            let foothold = self.footholds.get(&next_id)?;
            if !foothold.is_wall() && foothold.contains_x(x) {
                let ground = foothold.ground_at(x);
                if (ground.y - y).abs() <= MAX_CONNECTED_FOOTHOLD_Y_DELTA {
                    return Some(ground);
                }

                return None;
            }

            next_id = if x < f32::from(foothold.left()) {
                foothold.prev
            } else {
                foothold.next
            };
        }

        None
    }
}

#[derive(Clone, Copy)]
pub struct Ground {
    pub id: u16,
    pub y: f32,
    pub slope: f32,
    pub layer: u8,
}

#[derive(Clone, Copy)]
pub struct MapLine {
    pub id: u16,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

struct Foothold {
    id: u16,
    prev: u16,
    next: u16,
    layer: u8,
    x1: i16,
    y1: i16,
    x2: i16,
    y2: i16,
}

impl Foothold {
    fn from_node(node: NxNode<'_>, id: u16, layer: u8) -> Option<Self> {
        Some(Self {
            id,
            prev: node_integer(node, "prev").unwrap_or_default() as u16,
            next: node_integer(node, "next").unwrap_or_default() as u16,
            layer,
            x1: node_integer(node, "x1")? as i16,
            y1: node_integer(node, "y1")? as i16,
            x2: node_integer(node, "x2")? as i16,
            y2: node_integer(node, "y2")? as i16,
        })
    }

    fn left(&self) -> i16 {
        self.x1.min(self.x2)
    }

    fn right(&self) -> i16 {
        self.x1.max(self.x2)
    }

    fn top(&self) -> i16 {
        self.y1.min(self.y2)
    }

    fn bottom(&self) -> i16 {
        self.y1.max(self.y2)
    }

    fn is_wall(&self) -> bool {
        self.x1 == self.x2
    }

    fn slope(&self) -> f32 {
        if self.is_wall() {
            0.0
        } else {
            f32::from(self.y2 - self.y1) / f32::from(self.x2 - self.x1)
        }
    }

    fn ground_at(&self, x: f32) -> Ground {
        Ground {
            id: self.id,
            y: self.y_at(x),
            slope: self.slope(),
            layer: self.layer,
        }
    }

    fn y_at(&self, x: f32) -> f32 {
        if self.y1 == self.y2 {
            f32::from(self.y1)
        } else {
            self.slope() * (x - f32::from(self.x1)) + f32::from(self.y1)
        }
    }

    fn contains_x(&self, x: f32) -> bool {
        x >= f32::from(self.left()) && x <= f32::from(self.right())
    }
}

fn node_integer(node: NxNode<'_>, child: &str) -> Option<i64> {
    node.get(child).integer().ok().flatten()
}

fn map_info_ranges(map: NxNode<'_>) -> Option<(MapRange, MapRange)> {
    let info = map.get("info")?;
    let left = node_integer(info, "VRLeft")? as f32;
    let right = node_integer(info, "VRRight")? as f32;
    let top = node_integer(info, "VRTop")? as f32;
    let bottom = node_integer(info, "VRBottom")? as f32;

    Some((
        MapRange {
            min: left,
            max: right,
        },
        MapRange {
            min: top,
            max: bottom,
        },
    ))
}
