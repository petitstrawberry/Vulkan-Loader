# Native SWS window surfaces

The Scarlet loader accepts the experimental `VK_SGFX_scarlet_surface` instance
extension from supporting ICDs. It adds two instance-level lookups:

- `vkCreateScarletSurfaceSGFX`
- `vkGetPhysicalDeviceScarletPresentationSupportSGFX`

The private C ABI is declared in `loader/scarlet_wsi.h`; its numeric values are
not Khronos assignments. This extension depends on `VK_KHR_surface` and uses a
borrowed SWS window ID. The application retains the window until all swapchains
and surfaces that reference it have been destroyed.

Surfaces use ordinary loader-owned wrappers, allocation callbacks, per-ICD
lazy creation, and `vkDestroySurfaceKHR` cleanup. Calls pass through the instance
layer chain. Standard Vulkan surface queries and swapchain calls retain the
loader's usual surface unwrapping. Both the loader and ICD must support this
extension; it is not a substitute for another platform's WSI ABI.

The native SGFX ICD and Scarlet GLFW backend implement this contract. They use
one native `libsws_client_c.so` per process to share the SWS connection and
window IDs. The ICD remains an experimental, non-conformant Vulkan subset.
