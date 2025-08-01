use crate::character::{Character, ZMap};
use crate::map;
use crate::map::{Ladder, PortalType};
use crate::sound::play_sound;
use super::{State, Cooldown};
use glam::{vec2, Vec2};
use sdl3_sys::everything::*;
use std::sync::Arc;
use ::ui::geometry::Rect;
use ::ui::reactive::{RwSignal, SignalUpdate};
use ::ui::{geometry, input};
use crate::wz::Node;

#[derive(Default)]
pub struct Player {
    avatar: Character,
    direction: Vec2,
    speed: Vec2,
    position: Vec2,
    flip: bool,
    state: State,
    foothold: i32,
    layer: i32,
    ladder: Option<Ladder>,
    climb_cooldown: Cooldown,
}

impl Player {
    pub fn walk_force(&self) -> f32 {
        14000.0
    }
    pub fn walk_drag(&self) -> f32 {
        8000.0
    }
    pub fn walk_speed(&self) -> f32 {
        125.0
    }

    pub fn climb_speed(&self) -> f32 {
        100.0
    }

    pub fn gravity_acc(&self) -> f32 {
        2000.0
    }

    pub fn fall_speed(&self) -> f32 {
        670.0
    }

    pub fn jump_speed(&self) -> f32 {
        555.0
    }

    pub fn update(
        &mut self,
        map: &map::Map,
        current_map: Option<RwSignal<(String, Option<String>)>>,
        delta: f32,
    ) {
        self.step(map, current_map, delta);
        if !(self.speed == Vec2::ZERO && matches!(self.state, State::CLIMB)) {
            self.avatar.tick(delta);
        }
        self.climb_cooldown.update(delta);
    }

    pub fn jump(&mut self) {
        play_sound("Sound/Game.img/Jump");
        self.avatar.set_action("jump");
        self.state = State::FALL;
    }

    pub fn climb(&mut self, ladder: &Ladder) {
        self.speed = Vec2::ZERO;
        self.position.x = ladder.x;
        self.position.y = self.position.y.clamp(ladder.y1, ladder.y2);
        self.avatar
            .set_action(if ladder.is_ladder { "ladder" } else { "rope" });
        self.ladder = Some(ladder.clone());
        self.state = State::CLIMB;
    }

    pub fn step(
        &mut self,
        map: &map::Map,
        current_map: Option<RwSignal<(String, Option<String>)>>,
        delta: f32,
    ) -> f32 {
        let delta = delta / 1000.0;

        if matches!(self.state, State::CLIMB) {
            self.flip = false;
        } else if self.direction.x > 0.0 {
            self.flip = true;
        } else if self.direction.x < 0.0 {
            self.flip = false;
        }

        // jump left right
        if input::key_pressed(SDL_SCANCODE_LALT)
            && self.direction.y <= 0.0
            && self.direction.x != 0.0
            && matches!(
                self.state,
                State::WALK | State::STAND | State::PRONE | State::CLIMB
            )
        {
            if self.ladder.is_some() {
                self.climb_cooldown.set(200.0);
                self.ladder = None;
            }
            self.speed.x = self.walk_force() * 8.0 / 1000.0 * self.direction.x;
            self.speed.y = -self.jump_speed();
            self.jump();
            return 0.0;
        }

        // jump up
        if input::key_pressed(SDL_SCANCODE_LALT)
            && self.direction.y <= 0.0
            && self.direction.x == 0.0
            && matches!(self.state, State::WALK | State::STAND | State::PRONE)
        {
            self.speed.x = 0.0;
            self.speed.y = -self.jump_speed();
            self.jump();
            return 0.0;
        }

        // jump down
        if matches!(self.state, State::WALK | State::STAND | State::PRONE)
            && input::key_pressed(SDL_SCANCODE_LALT)
            && self.direction.y > 0.0
        {
            let foothold = map.footholds.values().find(|item| {
                !item.is_wall()
                    && item.id != self.foothold
                    && self.position.x >= item.start.x
                    && self.position.x <= item.end.x
                    && item.ground(self.position.x) > self.position.y
            });
            if let Some(_foothold) = foothold {
                self.position.y += 1.0;
                self.jump();
                return 0.0;
            }
        }

        // climb
        if matches!(self.state, State::WALK | State::STAND)
            && !self.climb_cooldown.value
            && self.direction.y != 0.0
        {
            let ladder = map.ladders.iter().find(|item| {
                let hor = self.position.x >= item.x - 10.0 && self.position.x < item.x + 10.0;
                let y = self.position.y + self.direction.y * 5.0;
                let ver = y >= item.y1 && y < item.y2;
                hor && ver
            });
            if let Some(ladder) = ladder {
                self.climb(ladder);
                return 0.0;
            }
        }

        if matches!(self.state, State::WALK | State::STAND | State::FALL)
            && input::key_pressed(SDL_SCANCODE_LCTRL)
        {
            self.avatar.set_action("swingO1");
        }

        if matches!(self.state, State::WALK | State::STAND | State::CLIMB)
            && self.direction.y < 0.0
            && current_map.is_some()
        {
            for portal in &map.portals {
                if !matches!(portal.pt, PortalType::REGULAR | PortalType::INVISIBLE) {
                    continue;
                }
                let lt = portal.position + vec2(-25.0, -100.0);
                let rb = portal.position + vec2(25.0, 25.0);
                let rect = Rect::from((lt, rb - lt));
                if rect.contains(self.position) {
                    current_map
                        .unwrap()
                        .set((format!("{:0>9}", portal.tm), Some(portal.tn.clone())));
                }
            }
        }

        match self.state {
            State::WALK => {
                if self.direction.y > 0.0 {
                    self.avatar.set_action("prone");
                    self.state = State::PRONE;
                    return 0.0;
                }

                if self.direction.x == 0.0 {
                    self.avatar.set_action("stand1");
                    self.state = State::STAND;
                    return 0.0;
                }

                let speed = self.speed.x
                    + self.direction.x * (self.walk_force() - self.walk_drag()) * delta;
                self.speed.x = speed.clamp(-self.walk_speed(), self.walk_speed());

                let x = self.position.x + self.speed.x * delta;

                let foothold = &map.footholds[&self.foothold];
                self.position.x = x.clamp(foothold.start.x, foothold.end.x);
                self.position.x = self.position.x.clamp(map.wall.left, map.wall.right);

                if !foothold.is_wall() {
                    self.position.y = foothold.ground(self.position.x);
                }

                let mut next = None;
                if self.direction.x < 0.0 && x < foothold.start.x {
                    next = Some(foothold.prev);
                }
                if self.direction.x > 0.0 && x > foothold.end.x {
                    next = Some(foothold.next);
                }

                if let Some(next) = next {
                    if next == 0 {
                        self.position.x = x;
                        self.avatar.set_action("jump");
                        self.state = State::FALL;
                    } else {
                        let foothold = &map.footholds[&next];
                        if foothold.is_blocking(self.position.y) {
                            return 0.0;
                        } else if foothold.is_wall() {
                            self.avatar.set_action("jump");
                            self.state = State::FALL;
                        } else {
                            self.foothold = foothold.id;
                        }
                    }
                }
            }
            State::STAND => {
                if self.direction.y > 0.0 {
                    self.avatar.set_action("prone");
                    self.state = State::PRONE;
                } else if self.direction.x != 0.0 {
                    self.avatar.set_action("walk1");
                    self.state = State::WALK;
                } else if self.speed.x != 0.0 {
                    let speed = self.speed.x - self.speed.x.signum() * self.walk_drag() * delta;
                    let speed = if self.speed.x > 0.0 {
                        speed.max(0.0)
                    } else {
                        speed.min(0.0)
                    };
                    self.speed.x = speed;
                    self.position += self.speed * delta;
                    self.position.x = self.position.x.clamp(map.wall.left, map.wall.right);
                }
            }
            State::FALL => {
                let prev = self.position;
                self.position += self.speed * delta;
                self.position.x = self.position.x.clamp(map.wall.left, map.wall.right);
                self.speed.y += self.gravity_acc() * delta;
                self.speed.y = self.speed.y.min(self.fall_speed());

                for foothold in map.footholds.values() {
                    if foothold.layer == self.layer
                        && (foothold.is_blocking(prev.y) || foothold.is_blocking(self.position.y))
                    {
                        if let Some(p) = geometry::intersect(
                            &foothold.start,
                            &foothold.end,
                            &prev,
                            &self.position,
                        ) {
                            self.position.x = p.x - self.direction.x * 1.0;
                            self.position.y = p.y;
                            self.speed.x = 0.0;
                            return 0.0;
                        }
                    }
                }

                if self.direction.y < 0.0 && !self.climb_cooldown.value {
                    for ladder in map.ladders.iter() {
                        if let Some(p) = geometry::intersect(
                            &vec2(ladder.x - 10.0, ladder.y1 - 5.0),
                            &vec2(ladder.x - 10.0, ladder.y2 + 5.0),
                            &prev,
                            &self.position,
                        )
                        .or_else(|| {
                            geometry::intersect(
                                &vec2(ladder.x + 10.0, ladder.y1 - 5.0),
                                &vec2(ladder.x + 10.0, ladder.y2 + 5.0),
                                &prev,
                                &self.position,
                            )
                        }) {
                            self.position.x = ladder.x;
                            self.position.y = p.y;
                            self.climb(ladder);
                            return 0.0;
                        }
                    }
                }

                if self.speed.y > 0.0 {
                    for foothold in map.footholds.values() {
                        if foothold.is_wall() {
                            continue;
                        }
                        if let Some(p) = geometry::intersect(
                            &foothold.start,
                            &foothold.end,
                            &prev,
                            &self.position,
                        ) {
                            self.position = p;
                            self.speed = Vec2::ZERO;
                            self.avatar.set_action("stand1");
                            self.foothold = foothold.id;
                            self.layer = foothold.layer;
                            self.state = State::STAND;
                            break;
                        }
                    }
                }
            }
            State::ALERT => {}
            State::PRONE => {
                if self.direction.y <= 0.0 {
                    self.avatar.set_action("stand1");
                    self.state = State::STAND;
                }
            }
            State::SWIM => {}
            State::CLIMB => {
                let ladder = self.ladder.unwrap();
                self.speed.y = self.direction.y * 100.0;
                self.position += self.speed * delta;

                if self.position.y < ladder.y1 {
                    if ladder.uf {
                        self.position.y = ladder.y1 - 5.0;
                        self.avatar.set_action("jump");
                        self.state = State::FALL;
                    } else {
                        self.position.y = self.position.y.max(ladder.y1);
                    }
                }
                if self.position.y > ladder.y2 {
                    self.position.y = ladder.y2;
                    self.avatar.set_action("jump");
                    self.state = State::FALL;
                }
            }
            State::DIED => {}
            State::SIT => {}
        }

        0.0
    }

    pub fn new(
        character_nodes: Vec<Node>,
        z_map: Arc<ZMap>,
        position: Vec2,
    ) -> Self {
        Self {
            avatar: Character::new(character_nodes, z_map),
            position,
            direction: Vec2::ZERO,
            speed: Vec2::ZERO,
            ..Default::default()
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn flip(&self) -> bool {
        self.flip
    }

    pub fn direction(&mut self) -> &mut Vec2 {
        &mut self.direction
    }

    pub fn frame(&self) -> Vec<crate::sprite::Sprite> {
        self.avatar.frame()
    }
}