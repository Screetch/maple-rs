use crate::WzSplitReaderContext;
use glam::Vec2;
use image::DynamicImage;
use sdl3_sys::everything::*;
use std::sync::Arc;
use ::ui::reactive::use_context;
use ui::reactive::{create_ref, Ref};
use ui::style::StyleTrigger;
use ::ui::style::Styleable;
use ui::taffy::{AvailableSpace, Size};
use ui::{use_resource, Element, Renderer, ViewId};

#[derive(Clone, Default)]
enum LoadState {
  #[default]
  Loading,
  Loaded(Arc<DynamicImage>),
  Failed(String),
}

/// 异步加载的 Image 组件
pub struct AsyncImage {
  id: ViewId,
  load_state: Ref<LoadState>,
}

impl AsyncImage {
  /// 创建新的异步图片组件
  pub fn new(path: impl Into<String>) -> Self {
    let id = ViewId::new();
    let reader = use_context::<WzSplitReaderContext>().unwrap().reader;
    let load_state = create_ref(LoadState::Loading);
    let path = path.into();
    use_resource(
      move || {
        {
          let reader = reader.clone();
          let path = path.clone();
          async move {
            match reader.get(&path).await {
              Ok(node_handle) => {
                // 从 NodeHandle 提取图片
                match Arc::<DynamicImage>::try_from(crate::wz::Node::from(&node_handle)) {
                  Ok(image) => LoadState::Loaded(image),
                  Err(_) => LoadState::Failed("Failed to extract image".to_string()),
                }
              }
              Err(_) => LoadState::Failed("Failed to load image".to_string()),
            }
          }
        }
      },
      move |state| {
        load_state.set(state);
        id.taffy().borrow_mut().mark_dirty(id.0).unwrap();
        id.request_repaint(StyleTrigger::Layout);
      },
    );

    Self { id, load_state }
  }
}

impl Element for AsyncImage {
  fn id(&self) -> ViewId {
    self.id
  }

  fn name(&self) -> String {
    "AsyncImage".to_string()
  }

  fn paint(&self, ctx: &mut Renderer) {
    let rect = self.id.layout_rect();
    self.load_state.with(move |state| {
      let LoadState::Loaded(image) = state else {
        return;
      };
      let texture = ctx.texture(image);
      ctx.render_texture_alpha(
        &texture,
        Vec2::ZERO,
        Vec2::ZERO,
        255,
        Some(rect.size()),
        SDL_FLIP_NONE,
      );
    });
  }

  fn measure(
    &self,
    ctx: &mut Renderer,
    known_dimensions: Size<Option<f32>>,
    _available_space: Size<AvailableSpace>,
  ) -> Size<f32> {
    self.load_state.with(move |state| {
      let LoadState::Loaded(image) = state else {
        return Size::ZERO;
      };
      let texture = ctx.texture(image);
      let image_size = texture.size;
      match (known_dimensions.width, known_dimensions.height) {
        (Some(width), Some(height)) => Size { width, height },
        (Some(width), None) => Size {
          width,
          height: (width / image_size.x) * image_size.y,
        },
        (None, Some(height)) => Size {
          width: (height / image_size.y) * image_size.x,
          height,
        },
        (None, None) => Size {
          width: image_size.x,
          height: image_size.y,
        },
      }
    })
  }
}

impl Styleable for AsyncImage {}