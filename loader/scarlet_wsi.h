/* SPDX-License-Identifier: Apache-2.0
 * Private native Scarlet WSI ABI shared with the GLFW and SGFX forks.
 * The structure/platform values are experimental, not Khronos assignments.
 */
#pragma once
#include <vulkan/vulkan.h>
#define VK_SGFX_SCARLET_SURFACE_EXTENSION_NAME "VK_SGFX_scarlet_surface"
#define VK_STRUCTURE_TYPE_SCARLET_SURFACE_CREATE_INFO_SGFX ((VkStructureType)0x7f530001)
#define VK_ICD_WSI_PLATFORM_SCARLET ((VkIcdWsiPlatform)0x53)
typedef struct VkScarletSurfaceCreateInfoSGFX {
    VkStructureType sType;
    const void *pNext;
    VkFlags flags;
    uint32_t window;
} VkScarletSurfaceCreateInfoSGFX;
typedef VkResult (VKAPI_PTR *PFN_vkCreateScarletSurfaceSGFX)(VkInstance, const VkScarletSurfaceCreateInfoSGFX *, const VkAllocationCallbacks *, VkSurfaceKHR *);
typedef VkBool32 (VKAPI_PTR *PFN_vkGetPhysicalDeviceScarletPresentationSupportSGFX)(VkPhysicalDevice, uint32_t);
VkResult VKAPI_CALL scarlet_surface_terminator(VkInstance, const VkScarletSurfaceCreateInfoSGFX *, const VkAllocationCallbacks *, VkSurfaceKHR *);
VkBool32 VKAPI_CALL scarlet_presentation_terminator(VkPhysicalDevice, uint32_t);
