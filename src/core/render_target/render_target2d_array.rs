#![allow(deprecated)]
use crate::core::render_target::*;

///
/// Adds additional functionality to write to and copy from both a [Texture2DArray]and
/// a [DepthTargetTexture2DArray] at the same time.
/// It purely adds functionality, so it can be created each time it is needed, the data is saved in the textures.
///
#[deprecated = "use RenderTarget instead"]
pub struct RenderTargetArray<'a, 'b> {
    context: Context,
    id: crate::context::Framebuffer,
    color_texture: Option<&'a mut Texture2DArray>,
    depth_texture: Option<&'b mut DepthTargetTexture2DArray>,
}
impl<'a, 'b> RenderTargetArray<'a, 'b> {
    ///
    /// Constructs a new render target that enables rendering into the given
    /// [DepthTargetTexture2DArray].
    ///
    pub fn new_depth(
        context: &Context,
        depth_texture: &'b mut DepthTargetTexture2DArray,
    ) -> ThreeDResult<Self> {
        Ok(Self {
            context: context.clone(),
            id: new_framebuffer(context)?,
            color_texture: None,
            depth_texture: Some(depth_texture),
        })
    }

    ///
    /// Constructs a new render target array that enables rendering into the given
    /// [Texture2DArray] and [DepthTargetTexture2DArray] array textures.
    ///
    pub fn new(
        context: &Context,
        color_texture: &'a mut Texture2DArray,
        depth_texture: &'b mut DepthTargetTexture2DArray,
    ) -> ThreeDResult<Self> {
        Ok(Self {
            context: context.clone(),
            id: new_framebuffer(context)?,
            color_texture: Some(color_texture),
            depth_texture: Some(depth_texture),
        })
    }

    ///
    /// Constructs a new render target array that enables rendering into the given
    /// [Texture2DArray].
    ///
    pub fn new_color(
        context: &Context,
        color_texture: &'a mut Texture2DArray,
    ) -> ThreeDResult<Self> {
        Ok(Self {
            context: context.clone(),
            id: new_framebuffer(context)?,
            color_texture: Some(color_texture),
            depth_texture: None,
        })
    }

    ///
    /// Renders whatever rendered in the `render` closure into the textures defined at construction
    /// and defined by the input parameters `color_layers` and `depth_layer`.
    /// Output at location *i* defined in the fragment shader is written to the color texture layer at the *ith* index in `color_layers`.
    /// The depth is written to the depth texture defined by `depth_layer`.
    /// Before writing, the textures are cleared based on the given clear state.
    ///
    pub fn write(
        &self,
        color_layers: &[u32],
        depth_layer: u32,
        clear_state: ClearState,
        render: impl FnOnce() -> ThreeDResult<()>,
    ) -> ThreeDResult<()> {
        self.bind(Some(color_layers), Some(depth_layer))?;
        clear(
            &self.context,
            &ClearState {
                red: self.color_texture.as_ref().and(clear_state.red),
                green: self.color_texture.as_ref().and(clear_state.green),
                blue: self.color_texture.as_ref().and(clear_state.blue),
                alpha: self.color_texture.as_ref().and(clear_state.alpha),
                depth: self.depth_texture.as_ref().and(clear_state.depth),
            },
        );
        render()?;
        if let Some(ref color_texture) = self.color_texture {
            color_texture.generate_mip_maps();
        }
        Ok(())
    }

    fn bind(&self, color_layers: Option<&[u32]>, depth_layer: Option<u32>) -> ThreeDResult<()> {
        unsafe {
            self.context
                .bind_framebuffer(crate::context::DRAW_FRAMEBUFFER, Some(self.id));
            if let Some(ref color_texture) = self.color_texture {
                if let Some(color_layers) = color_layers {
                    self.context.draw_buffers(
                        &(0..color_layers.len())
                            .map(|i| crate::context::COLOR_ATTACHMENT0 + i as u32)
                            .collect::<Vec<u32>>(),
                    );
                    for channel in 0..color_layers.len() {
                        color_texture.bind_as_color_target(
                            color_layers[channel],
                            channel as u32,
                            0,
                        );
                    }
                }
            }
        }
        if let Some(ref depth_texture) = self.depth_texture {
            if let Some(depth_layer) = depth_layer {
                depth_texture.bind_as_depth_target(depth_layer);
            }
        }
        self.context.framebuffer_check()?;
        self.context.error_check()
    }
}

impl Drop for RenderTargetArray<'_, '_> {
    fn drop(&mut self) {
        unsafe {
            self.context.delete_framebuffer(self.id);
        }
    }
}

fn clear(context: &Context, clear_state: &ClearState) {
    context.set_write_mask(WriteMask {
        red: clear_state.red.is_some(),
        green: clear_state.green.is_some(),
        blue: clear_state.blue.is_some(),
        alpha: clear_state.alpha.is_some(),
        depth: clear_state.depth.is_some(),
    });
    let clear_color = clear_state.red.is_some()
        || clear_state.green.is_some()
        || clear_state.blue.is_some()
        || clear_state.alpha.is_some();
    unsafe {
        if clear_color {
            context.clear_color(
                clear_state.red.unwrap_or(0.0),
                clear_state.green.unwrap_or(0.0),
                clear_state.blue.unwrap_or(0.0),
                clear_state.alpha.unwrap_or(1.0),
            );
        }
        if let Some(depth) = clear_state.depth {
            context.clear_depth_f32(depth);
        }
        context.clear(if clear_color && clear_state.depth.is_some() {
            crate::context::COLOR_BUFFER_BIT | crate::context::DEPTH_BUFFER_BIT
        } else {
            if clear_color {
                crate::context::COLOR_BUFFER_BIT
            } else {
                crate::context::DEPTH_BUFFER_BIT
            }
        });
    }
}
