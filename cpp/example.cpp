#include <SDL2/SDL.h>
#include <LibGfx/LibGfxRust.h>

#include <sys/stat.h>
#include <sys/mman.h>
#include <errno.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/types.h>

SDL_PixelFormatEnum bitmap_format_to_sdl_pixel_format(FFI::BitmapFormat format)
{
    switch (format) {
    case FFI::BitmapFormat::BGRx8888:
        return SDL_PIXELFORMAT_BGRX8888;
    case FFI::BitmapFormat::BGRA8888:
        return SDL_PIXELFORMAT_BGRA8888;
    case FFI::BitmapFormat::RGBA8888:
        return SDL_PIXELFORMAT_RGBA8888;
    default:
        return SDL_PIXELFORMAT_UNKNOWN;
    }
}

int main(int argc, char *argv[])
{

    if (SDL_Init(SDL_INIT_VIDEO) != 0) {
        SDL_Log("Unable to initialize SDL: %s", SDL_GetError());
        return 1;
    }

    if (argc < 2) {
        SDL_Log("Usage: %s <image>", argv[0]);
        return 1;
    }

    int fd = open(argv[1], O_RDONLY);
    if (fd < 0) {
        SDL_Log("Unable to open file: %s", strerror(errno));
        return 1;
    }

    struct stat fd_stats = {};
    if (fstat(fd, &fd_stats) < 0) {
        SDL_Log("Unable to stat file: %s", strerror(errno));
        return 1;
    }

    uint8_t* buffer = static_cast<uint8_t*>(mmap(nullptr, fd_stats.st_size, PROT_READ, MAP_PRIVATE, fd, 0));
    if (buffer == MAP_FAILED) {
        SDL_Log("Unable to mmap file: %s", strerror(errno));
        return 1;
    }

    SDL_Window *window = SDL_CreateWindow("SDL2 Window",
                                          SDL_WINDOWPOS_CENTERED,
                                          SDL_WINDOWPOS_CENTERED,
                                          800, 600,
                                          SDL_WINDOW_HIDDEN);
    SDL_Renderer *renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED);

    auto decoder = FFI::tga_image_decoder_plugin_new(buffer, fd_stats.st_size);
    if (!decoder) {
        SDL_Log("Unable to create decoder");
        return 1;
    }

    SDL_ShowWindow(window);

    auto frame_count = FFI::image_decoder_plugin_frame_count(decoder);
    //auto frame_count = size_t(1);
    printf("got decoder %p with frame count %zu\n", decoder, frame_count);

    for (size_t i = 0; i < frame_count; i++) {
        auto frame = FFI::image_decoder_plugin_frame(decoder, i);
        auto depth = FFI::bitmap_format_bytes_per_pixel(frame.image.format);
        printf("got frame %zu with depth %d\n", i, depth);
        printf("frame size: %dx%d\n", frame.image.size.width, frame.image.size.height);
        printf("pixel format: %d\n", static_cast<int>(frame.image.format));
        printf("pitch: %d\n", frame.image.pitch);
        printf("SDL pixel format: %d\n", bitmap_format_to_sdl_pixel_format(frame.image.format));
        printf("SDL_PIXELFORMAT_BGRX8888: %d\n", SDL_PIXELFORMAT_BGRX8888);

        SDL_Surface* surface = SDL_CreateRGBSurfaceWithFormatFrom(frame.image.data.data,
                                                         frame.image.size.width,
                                                         frame.image.size.height,
                                                         depth * 8,
                                                         frame.image.pitch,
                                                         bitmap_format_to_sdl_pixel_format(frame.image.format));

        SDL_SetWindowSize(window, frame.image.size.width, frame.image.size.height);
        SDL_Texture* texture = SDL_CreateTextureFromSurface(renderer, surface);
        SDL_RenderCopy(renderer, texture, nullptr, nullptr);
        SDL_RenderPresent(renderer);

        SDL_UpdateWindowSurface(window);

        SDL_Delay(5000);

        SDL_DestroyTexture(texture);
        SDL_FreeSurface(surface);
        FFI::image_decoder_plugin_free_frame(frame);
    }

    FFI::tga_image_decoder_plugin_free(decoder);
    SDL_DestroyRenderer(renderer);
    SDL_DestroyWindow(window);
    munmap(buffer, fd_stats.st_size);

    SDL_Quit();
    return 0;
}
