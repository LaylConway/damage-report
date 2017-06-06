use std::sync::{Arc};
use std::time::{Duration};

use cgmath::{Vector2};
use slog::{Logger};
use vulkano::format::{Format};
use vulkano::buffer::{CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferBuilder};
use vulkano::device::{DeviceExtensions, Device, Queue};
use vulkano::framebuffer::{Framebuffer, RenderPassAbstract, FramebufferAbstract};
use vulkano::format::{D16Unorm};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::swapchain::{Swapchain, SurfaceTransform};
use vulkano::image::{SwapchainImage};
use vulkano::image::immutable::{ImmutableImage};
use vulkano::sync::{GpuFuture};
use vulkano_win::{self, VkSurfaceBuild, Window};
use winit::{EventsLoop, WindowBuilder, Event as WinitEvent, WindowEvent, ElementState, ScanCode, VirtualKeyCode, ModifiersState};

use error::{CobaltErrorMap};
use {Error};

/// Manages the window/screen to render to, and which GPU to render with. Also manages generic
/// tasks needed for rendering smoothly to the target regardless of game-specific render behavior,
/// such as v-sync and doube/triplebuffering.
pub struct Target {
    // Winit window
    events_loop: EventsLoop,
    window: Window,

    // Persistent values needed for vulkan rendering
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    images: Vec<Arc<SwapchainImage>>,
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,

    // Submissions from previous frames
    submissions: Vec<Box<GpuFuture>>,

    // Queued up things we need to submit as part of command buffers
    queued_texture_copies: Vec<(
        Arc<CpuAccessibleBuffer<[u8]>>,
        Arc<ImmutableImage<Format>>
    )>,

    // Generic data
    size: Vector2<u32>,
    focused: bool,
}

impl Target {
    pub fn new(log: &Logger) -> Result<Self, Error> {
        info!(log, "Initializing target"; "backend" => "vulkan");
        let size = Vector2::new(1280, 720);

        // Start by setting up the vulkano instance, this is a silo of vulkan that all our vulkan
        //  types will belong to
        debug!(log, "Creating Vulkan instance");
        let instance = {
            // Tell it we need at least the extensions vulkano-win needs
            let extensions = vulkano_win::required_extensions();
            Instance::new(None, &extensions, None)
                .map_platform_err()?
        };

        // Pick a GPU to use for rendering. We assume first device as the one to render with
        // TODO: Allow user to select in some way, perhaps through config
        debug!(log, "Finding target physical device");
        let physical = PhysicalDevice::enumerate(&instance).next()
            .ok_or_else(|| Error::Platform("No physical devices found".into()))?;
        debug!(log, "Found physical device";
            "device" => physical.name(), "type" => format!("{:?}", physical.ty())
        );

        // Set up the window we want to render to, along with an EventsLoop we can use to listen
        //  for input and other events happening to the window coming from the OS
        debug!(log, "Creating window");
        let events_loop = EventsLoop::new();
        let window = WindowBuilder::new()
            .with_dimensions(size.x, size.y)
            .with_title(format!("Cobalt"))
            .build_vk_surface(&events_loop, instance.clone())
            .map_platform_err()?;

        // Find a GPU graphics queue family, we later create a queue from this family to talk to
        //  the GPU
        debug!(log, "Finding graphics queue family with required features");
        let graphics_queue_family = physical.queue_families().find(|q| {
            // The queue needs to support graphics (of course) and needs to support drawing to
            //  the previously created window's surface
            q.supports_graphics() && window.surface().is_supported(*q).unwrap_or(false)
        }).unwrap();

        // Finally, we create our actual connection with the GPU. We need a "device", which
        //  represents the connection between our program and the device, and queues, which we use
        //  to issue rendering commands to the GPU
        debug!(log, "Creating logical device and queues");
        let (device, mut queues) = {
            // We need to request features explicitly, we need at least the swap chain
            let device_ext = DeviceExtensions {
                khr_swapchain: true,
                .. DeviceExtensions::none()
            };

            // Create the actual device
            Device::new(
                &physical, physical.supported_features(), &device_ext,
                // Pass which queues we want, we want one single graphics queue, the priority
                //  doesn't really matter to us since there's only one
                [(graphics_queue_family, 0.5)].iter().cloned()
            ).unwrap()
        };

        // Get the graphics queue we requested
        let graphics_queue = queues.next().unwrap();

        // Now create the swapchain, we need this to actually swap between our back buffer and the
        //  window's front buffer, without it we can't show anything
        debug!(log, "Creating swapchain");
        let (swapchain, images) = {
            // Get what the swap chain we want to create would be capable of, we can't request
            //  anything it can't do
            let caps = window.surface().capabilities(physical).unwrap();

            // The swap chain's dimensions need to match the window size
            let dimensions = caps.current_extent.unwrap_or([size.x, size.y]);

            // The present mode is things like vsync and vsync-framerate, right now pick the first
            //  one, we're sure it will work but it's probably not optimal
            // TODO: Let the user decide
            let present = caps.present_modes.iter().next().unwrap();

            // This decides how alpha will be composited by the OS' window manager, we just pick
            //  the first available option
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();

            // And finally, chose the internal format that images will have
            // The swap chain needs to be in SRGB, and this format is guaranteed supported
            let format = ::vulkano::format::B8G8R8A8Srgb;

            // Finally, actually create the swap chain, with all its color images
            Swapchain::new(
                device.clone(), window.surface().clone(), caps.min_image_count, format,
                dimensions, 1,
                caps.supported_usage_flags, &graphics_queue, SurfaceTransform::Identity, alpha,
                present, true, None
            ).unwrap()
        };

        // To render in 3D, we need an extra buffer to keep track of the depth. Since this won't be
        //  displayed, it doesn't need to be part of the swapchain.
        debug!(log, "Creating depth buffer");
        let depth_buffer = {
            use vulkano::image::{Image};
            use vulkano::image::attachment::{AttachmentImage};

            AttachmentImage::transient(
                device.clone(), images[0].dimensions().width_height(), D16Unorm
            ).unwrap()
        };

        // Set up a render pass TODO: Comment better
        let color_buffer_format = swapchain.format();
        let depth_buffer_format = ::vulkano::format::Format::D16Unorm;
        #[allow(dead_code)]
        let render_pass = Arc::new(single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: color_buffer_format,
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: depth_buffer_format,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        ).unwrap());

        // Set up the frame buffers matching the render pass
        // For each image in the swap chain, we create a frame buffer that renders to that image
        //  and to the depth buffer attachment. These attachments are defined by the render pass.
        debug!(log, "Creating framebuffers for swapchain");
        let framebuffers = images.iter().map(|image| {
            Arc::new(Framebuffer::start(render_pass.clone())
                .add(image.clone()).unwrap()
                .add(depth_buffer.clone()).unwrap()
                .build().unwrap()
            ) as Arc<FramebufferAbstract + Send + Sync>
        }).collect::<Vec<_>>();

        Ok(Target {
            events_loop,
            window,

            device,
            graphics_queue,
            images,
            swapchain,
            render_pass,
            framebuffers,

            submissions: Vec::new(),

            queued_texture_copies: Vec::new(),

            size,
            focused: true,
        })
    }

    pub fn poll_events(&mut self) -> Vec<Event> {
        let mut event = Vec::new();
        let focused = &mut self.focused;

        self.events_loop.poll_events(|ev| {
            match ev {
                WinitEvent::WindowEvent { event: ev, .. } => {
                    match ev {
                        WindowEvent::Closed => event.push(Event::Closed),
                        WindowEvent::Focused(efocused) => *focused = efocused,
                        WindowEvent::KeyboardInput(state, scan_code, key_code, modifiers) =>
                            event.push(
                                Event::KeyboardInput(state, scan_code, key_code, modifiers)
                            ),
                        WindowEvent::MouseMoved(x, y) =>
                            if *focused {
                                event.push(Event::MouseMoved(Vector2::new(x as u32, y as u32)))
                            },
                        _ => (),
                    }
                },
            }
        });

        event
    }

    pub fn start_frame(&mut self) -> Frame {
        // Clearing the old submissions by keeping alive only the ones which probably aren't
        //  finished
        while self.submissions.len() >= 4 {
            self.submissions.remove(0);
        }

        // Get the image for this frame
        let (image_num, future) = ::vulkano::swapchain::acquire_next_image(
            self.swapchain.clone(), Duration::new(1, 0)
        ).unwrap();
        let mut future: Box<GpuFuture> = Box::new(future);

        // If we have any images to load, we need to submit another buffer before anything else
        if self.queued_texture_copies.len() != 0 {
            // Create a command buffer to upload the textures with
            let mut image_copy_buffer_builder = AutoCommandBufferBuilder::new(
                self.device.clone(), self.graphics_queue.family()
            ).unwrap();

            // Add any textures we need to upload to the command buffer
            while let Some(val) = self.queued_texture_copies.pop() {
                image_copy_buffer_builder = image_copy_buffer_builder
                    .copy_buffer_to_image(val.0, val.1)
                    .unwrap();
            }

            // Finally, add the command buffer to the future so it will be executed
            let image_copy_buffer = image_copy_buffer_builder.build().unwrap();
            future = Box::new(future
                .then_execute(self.graphics_queue.clone(), image_copy_buffer).unwrap()
            );
        }

        Frame {
            framebuffer: self.framebuffers[image_num].clone(),
            image_num,
            future: Some(future),
        }
    }

    pub fn finish_frame(&mut self, mut frame: Frame) {
        let future = frame.future.take().unwrap()
            // Present the image resulting from all the submitted command buffers on the screen
            .then_swapchain_present(
                self.graphics_queue.clone(), self.swapchain.clone(), frame.image_num
            )
            // Finally, submit it all to the driver
            .then_signal_fence_and_flush().unwrap();

        // Keep track of these submissions so we can later wait on them
        self.submissions.push(Box::new(future));
    }

    pub fn set_cursor_position(&self, position: Vector2<u32>) {
        self.window.window()
            .set_cursor_position(position.x as i32, position.y as i32)
            .unwrap();
    }

    pub fn queue_texture_copy(
        &mut self,
        buffer: Arc<CpuAccessibleBuffer<[u8]>>,
        texture: Arc<ImmutableImage<Format>>,
    ) {
        self.queued_texture_copies.push((buffer, texture));
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn graphics_queue(&self) -> &Arc<Queue> {
        &self.graphics_queue
    }

    pub fn images(&self) -> &Vec<Arc<SwapchainImage>> {
        &self.images
    }

    pub fn render_pass(&self) -> &Arc<RenderPassAbstract + Send + Sync> {
        &self.render_pass
    }

    pub fn size(&self) -> Vector2<u32> {
        self.size
    }
}

#[derive(Debug)]
pub enum Event {
    Closed,
    KeyboardInput(ElementState, ScanCode, Option<VirtualKeyCode>, ModifiersState),
    MouseMoved(Vector2<u32>),
}

pub struct Frame {
    pub framebuffer: Arc<FramebufferAbstract + Send + Sync>,
    image_num: usize,
    pub future: Option<Box<GpuFuture>>,
}
