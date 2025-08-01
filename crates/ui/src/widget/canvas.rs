use crate::element::Element;
use crate::event::Interactive;
use crate::geometry::Rect;
use crate::render::renderer::{NineGridTexture, Renderer, Texture};
use crate::style::{StyleTrigger, Styleable};
use crate::view_id::ViewId;
use glam::{vec2, Vec2};
use peniko::Color;
use reactive::{create_effect, create_ref, Ref};
use sdl3_sys::everything::SDL_FlipMode;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use taffy::{AvailableSpace, Size};

#[derive(Clone)]
pub enum DrawCommand {
  FillRect {
    color: Color,
    rect: Rect,
  },
  StrokeLine {
    color: Color,
    p1: Vec2,
    p2: Vec2,
  },
  FillCircle {
    color: Color,
    center: Vec2,
    radius: f32,
  },
  StrokeRect {
    color: Color,
    rect: Rect,
  },
  StrokeCircle {
    color: Color,
    center: Vec2,
    radius: f32,
  },
  DrawTexture {
    texture: Arc<Texture>,
    src_rect: Option<Rect>,
    dst_rect: Rect,
    color: Option<Color>,
  },
  DrawTextureRotated {
    texture: Arc<Texture>,
    src_rect: Rect,
    dst_rect: Rect,
    angle: f64,
    center: Option<Vec2>,
    flip: SDL_FlipMode,
  },
  DrawTextureAlpha {
    texture: Arc<Texture>,
    position: Vec2,
    origin: Vec2,
    alpha: f32,
    size: Option<Vec2>,
    flip: SDL_FlipMode,
  },
  DrawTextureNineGrid {
    texture: Arc<NineGridTexture>,
    position: Vec2,
    size: Option<Vec2>,
  },
}

pub struct CanvasContext {
  commands: Vec<DrawCommand>,
}

impl CanvasContext {
  pub fn new() -> Self {
    Self {
      commands: Vec::new(),
    }
  }

  pub fn fill_rect(&mut self, color: Color, rect: Rect) {
    self.commands.push(DrawCommand::FillRect { color, rect });
  }

  pub fn stroke_line(&mut self, color: Color, p1: Vec2, p2: Vec2) {
    self
      .commands
      .push(DrawCommand::StrokeLine { color, p1, p2 });
  }

  pub fn fill_circle(&mut self, color: Color, center: Vec2, radius: f32) {
    self.commands.push(DrawCommand::FillCircle {
      color,
      center,
      radius,
    });
  }

  pub fn stroke_rect(&mut self, color: Color, rect: Rect) {
    self.commands.push(DrawCommand::StrokeRect { color, rect });
  }

  pub fn stroke_circle(&mut self, color: Color, center: Vec2, radius: f32) {
    self.commands.push(DrawCommand::StrokeCircle {
      color,
      center,
      radius,
    });
  }

  // 基础纹理绘制
  pub fn draw_texture(&mut self, texture: &Arc<Texture>, dst_rect: Rect) {
    self.commands.push(DrawCommand::DrawTexture {
      texture: texture.clone(),
      src_rect: None,
      dst_rect,
      color: None,
    });
  }

  // 带源矩形的纹理绘制
  pub fn draw_texture_rect(&mut self, texture: &Arc<Texture>, src_rect: Rect, dst_rect: Rect) {
    self.commands.push(DrawCommand::DrawTexture {
      texture: texture.clone(),
      src_rect: Some(src_rect),
      dst_rect,
      color: None,
    });
  }

  // 带颜色调制的纹理绘制
  pub fn draw_texture_tinted(&mut self, texture: &Arc<Texture>, dst_rect: Rect, color: Color) {
    self.commands.push(DrawCommand::DrawTexture {
      texture: texture.clone(),
      src_rect: None,
      dst_rect,
      color: Some(color),
    });
  }

  // 完整参数的纹理绘制
  pub fn draw_texture_full(
    &mut self,
    texture: &Arc<Texture>,
    src_rect: Option<Rect>,
    dst_rect: Rect,
    color: Option<Color>,
  ) {
    self.commands.push(DrawCommand::DrawTexture {
      texture: texture.clone(),
      src_rect,
      dst_rect,
      color,
    });
  }

  // 旋转纹理绘制
  pub fn draw_texture_rotated(
    &mut self,
    texture: &Arc<Texture>,
    dst_rect: Rect,
    angle: f64,
    center: Option<Vec2>,
  ) {
    self.commands.push(DrawCommand::DrawTextureRotated {
      texture: texture.clone(),
      src_rect: rect(0.0, 0.0, texture.size.x, texture.size.y),
      dst_rect,
      angle,
      center,
      flip: SDL_FlipMode::NONE,
    });
  }

  // 带翻转的旋转纹理绘制
  pub fn draw_texture_rotated_flipped(
    &mut self,
    texture: &Arc<Texture>,
    src_rect: Rect,
    dst_rect: Rect,
    angle: f64,
    center: Option<Vec2>,
    flip: SDL_FlipMode,
  ) {
    self.commands.push(DrawCommand::DrawTextureRotated {
      texture: texture.clone(),
      src_rect,
      dst_rect,
      angle,
      center,
      flip,
    });
  }

  // 透明度纹理绘制
  pub fn draw_texture_alpha(&mut self, texture: &Arc<Texture>, position: Vec2, alpha: f32) {
    self.commands.push(DrawCommand::DrawTextureAlpha {
      texture: texture.clone(),
      position,
      origin: Vec2::ZERO,
      alpha,
      size: None,
      flip: SDL_FlipMode::NONE,
    });
  }

  // 完整参数的透明度纹理绘制
  pub fn draw_texture_alpha_full(
    &mut self,
    texture: &Arc<Texture>,
    position: Vec2,
    origin: Vec2,
    alpha: f32,
    size: Option<Vec2>,
    flip: SDL_FlipMode,
  ) {
    self.commands.push(DrawCommand::DrawTextureAlpha {
      texture: texture.clone(),
      position,
      origin,
      alpha,
      size,
      flip,
    });
  }

  // 翻转纹理绘制
  pub fn draw_texture_flipped(
    &mut self,
    texture: &Arc<Texture>,
    dst_rect: Rect,
    flip: SDL_FlipMode,
  ) {
    self.commands.push(DrawCommand::DrawTextureRotated {
      texture: texture.clone(),
      src_rect: rect(0.0, 0.0, texture.size.x, texture.size.y),
      dst_rect,
      angle: 0.0,
      center: None,
      flip,
    });
  }

  // 九宫格纹理绘制
  pub fn draw_texture_nine_grid(
    &mut self,
    texture: &Arc<NineGridTexture>,
    position: Vec2,
    size: Option<Vec2>,
  ) {
    self.commands.push(DrawCommand::DrawTextureNineGrid {
      texture: texture.clone(),
      position,
      size,
    });
  }

  pub fn bounds(&self) -> Rect {
    // 占位符，实际使用时由 Canvas 提供
    Rect {
      x: 0.0,
      y: 0.0,
      width: 0.0,
      height: 0.0,
    }
  }

  pub fn width(&self) -> f32 {
    self.bounds().width
  }

  pub fn height(&self) -> f32 {
    self.bounds().height
  }
}

pub struct Canvas {
  id: ViewId,
  draw_commands: Rc<RefCell<Vec<DrawCommand>>>,
  size_fn: Ref<Option<Box<dyn Fn(Size<Option<f32>>, Size<AvailableSpace>) -> Size<f32>>>>,
}

pub trait CanvasSize {
  fn into_size_fn(&self) -> Box<dyn Fn(Size<Option<f32>>, Size<AvailableSpace>) -> Size<f32>>;
}

impl CanvasSize for Size<f32> {
  fn into_size_fn(&self) -> Box<dyn Fn(Size<Option<f32>>, Size<AvailableSpace>) -> Size<f32>> {
    let size = self.clone();
    Box::new(move |_, _| size)
  }
}

impl CanvasSize for (f32, f32) {
  fn into_size_fn(&self) -> Box<dyn Fn(Size<Option<f32>>, Size<AvailableSpace>) -> Size<f32>> {
    let size = Size {
      width: self.0,
      height: self.1,
    };
    Box::new(move |_, _| size)
  }
}

impl CanvasSize for f32 {
  fn into_size_fn(&self) -> Box<dyn Fn(Size<Option<f32>>, Size<AvailableSpace>) -> Size<f32>> {
    let size = self.clone();
    Box::new(move |_, _| Size {
      width: size,
      height: size,
    })
  }
}

pub struct AspectRatio(pub f32);

impl CanvasSize for AspectRatio {
  fn into_size_fn(&self) -> Box<dyn Fn(Size<Option<f32>>, Size<AvailableSpace>) -> Size<f32>> {
    let ratio = self.0;
    Box::new(move |known, _| match (known.width, known.height) {
      (Some(w), _) => Size {
        width: w,
        height: w / ratio,
      },
      (_, Some(h)) => Size {
        width: h * ratio,
        height: h,
      },
      _ => Size {
        width: 200.0,
        height: 200.0 / ratio,
      },
    })
  }
}

impl<F> CanvasSize for F
where
  F: Fn() -> Box<dyn Fn(Size<Option<f32>>, Size<AvailableSpace>) -> Size<f32>> + 'static,
{
  fn into_size_fn(&self) -> Box<dyn Fn(Size<Option<f32>>, Size<AvailableSpace>) -> Size<f32>> {
    Box::new(self())
  }
}

pub fn aspect_ratio(ratio: f32) -> AspectRatio {
  AspectRatio(ratio)
}

pub fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
  Rect {
    x,
    y,
    width,
    height,
  }
}

// 翻转模式常量
pub const FLIP_NONE: SDL_FlipMode = SDL_FlipMode::NONE;
pub const FLIP_HORIZONTAL: SDL_FlipMode = SDL_FlipMode::HORIZONTAL;
pub const FLIP_VERTICAL: SDL_FlipMode = SDL_FlipMode::VERTICAL;

impl Canvas {
  pub fn new<F>(draw_fn: F) -> Self
  where
    F: Fn(&mut CanvasContext) + 'static,
  {
    let id = ViewId::new();
    let draw_commands = Rc::new(RefCell::new(Vec::new()));

    create_effect({
      let draw_commands = draw_commands.clone();
      move |_| {
        let mut ctx = CanvasContext::new();
        draw_fn(&mut ctx);
        *draw_commands.borrow_mut() = ctx.commands;
        id.request_repaint(StyleTrigger::Paint);
      }
    });

    Self {
      id,
      draw_commands,
      size_fn: create_ref(None),
    }
  }

  pub fn size<S: CanvasSize + 'static>(mut self, size: S) -> Self {
    let size_fn = self.size_fn;
    let aa = create_ref(size);
    create_effect(move |_| {
      size_fn.with_mut(move |size_fn| {
        aa.with(move |a| {
          *size_fn = Some(Box::new(a.into_size_fn()));
        })
      })
    });
    self
  }
}

impl Element for Canvas {
  fn id(&self) -> ViewId {
    self.id
  }

  fn name(&self) -> String {
    "Canvas".to_string()
  }

  fn paint(&self, renderer: &mut Renderer) {
    let layout = self.id().layout();
    let bounds = Rect {
      x: 0.0,
      y: 0.0,
      width: layout.size.width,
      height: layout.size.height,
    };

    for command in self.draw_commands.borrow().iter() {
      match command {
        DrawCommand::FillRect { color, rect } => {
          let absolute_rect = Rect {
            x: bounds.x + rect.x,
            y: bounds.y + rect.y,
            width: rect.width,
            height: rect.height,
          };
          renderer.fill_rect(*color, absolute_rect);
        }
        DrawCommand::StrokeLine { color, p1, p2 } => {
          let abs_p1 = vec2(bounds.x + p1.x, bounds.y + p1.y);
          let abs_p2 = vec2(bounds.x + p2.x, bounds.y + p2.y);
          renderer.line(*color, abs_p1, abs_p2);
        }
        DrawCommand::StrokeRect { color, rect } => {
          let absolute_rect = Rect {
            x: bounds.x + rect.x,
            y: bounds.y + rect.y,
            width: rect.width,
            height: rect.height,
          };
          renderer.stroke_rect(*color, absolute_rect);
        }
        DrawCommand::FillCircle {
          color,
          center,
          radius,
        } => {
          // 使用矩形近似圆形，实际实现可能需要更复杂的圆形绘制
          let circle_rect = Rect {
            x: bounds.x + center.x - radius,
            y: bounds.y + center.y - radius,
            width: radius * 2.0,
            height: radius * 2.0,
          };
          renderer.fill_rect(*color, circle_rect);
        }
        DrawCommand::StrokeCircle {
          color,
          center,
          radius,
        } => {
          let circle_rect = Rect {
            x: bounds.x + center.x - radius,
            y: bounds.y + center.y - radius,
            width: radius * 2.0,
            height: radius * 2.0,
          };
          renderer.stroke_rect(*color, circle_rect);
        }
        DrawCommand::DrawTexture {
          texture,
          src_rect,
          dst_rect,
          color,
        } => {
          let absolute_dst = Rect {
            x: bounds.x + dst_rect.x,
            y: bounds.y + dst_rect.y,
            width: dst_rect.width,
            height: dst_rect.height,
          };
          let src = src_rect.unwrap_or(Rect::new(0.0, 0.0, texture.size.x, texture.size.y));
          renderer.render_texture(texture.texture, src, absolute_dst, *color, false);
        }
        DrawCommand::DrawTextureRotated {
          texture,
          src_rect,
          dst_rect,
          angle,
          center,
          flip,
        } => {
          let absolute_dst = Rect {
            x: bounds.x + dst_rect.x,
            y: bounds.y + dst_rect.y,
            width: dst_rect.width,
            height: dst_rect.height,
          };
          let absolute_center = center.map(|c| vec2(bounds.x + c.x, bounds.y + c.y));
          renderer.render_texture_rotated(
            texture,
            *src_rect,
            absolute_dst,
            *angle,
            absolute_center,
            *flip,
          );
        }
        DrawCommand::DrawTextureAlpha {
          texture,
          position,
          origin,
          alpha,
          size,
          flip,
        } => {
          let absolute_pos = vec2(bounds.x + position.x, bounds.y + position.y);
          let absolute_origin = vec2(bounds.x + origin.x, bounds.y + origin.y);
          renderer.render_texture_alpha(
            texture,
            absolute_pos,
            absolute_origin,
            (*alpha * 255.0) as i32,
            *size,
            *flip,
          );
        }
        DrawCommand::DrawTextureNineGrid {
          texture,
          position,
          size,
        } => {
          let absolute_pos = vec2(bounds.x + position.x, bounds.y + position.y);
          renderer.render_texture_nine_grid(texture, absolute_pos, *size);
        }
      }
    }
  }

  fn measure(
    &self,
    _ctx: &mut Renderer,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
  ) -> Size<f32> {
    self.size_fn.with(|size_fn| {
      if let Some(size_fn) = size_fn {
        size_fn(known_dimensions, available_space)
      } else {
        Size::ZERO
      }
    })
  }
}

impl Interactive for Canvas {}
impl Styleable for Canvas {}
