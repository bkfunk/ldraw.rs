# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LDraw.rs is a Rust library for manipulating and rendering LDraw (virtual LEGO CAD) model files. It can be compiled to WebAssembly for browser-based rendering.

## Build Commands

### Core Library
```bash
# Build all crates
cargo build

# Build in release mode
cargo build --release

# Build specific crate
cargo build -p ldraw
cargo build -p ldraw-ir
cargo build -p ldraw-renderer
cargo build -p ldraw-olr

# Run tests
cargo test
cargo test -p ldraw  # Test specific crate
```

### Tools
```bash
# Build command-line tools
cargo build -p baker
cargo build -p ldr2img
cargo build -p viewer_native

# Run ldr2img tool (example)
cargo run -p ldr2img -- input.ldr output.png
```

### Web Viewer (WASM)
```bash
cd tools/viewer/web

# Install npm dependencies
npm install

# Build WASM module and bundle
npm run build

# Development server
npm run serve
```

## Architecture

The project follows a layered architecture with clear separation of concerns:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   ldraw     │ ──> │     ir      │ ──> │  renderer   │ <── │    olr      │
│             │     │             │     │             │     │             │
│ File Format │     │  Geometry   │     │    GPU      │     │  Headless   │
│  Parsing    │     │ Processing  │     │ Rendering   │     │ Rendering   │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

### Core Crates

- **`ldraw`**: Base library for parsing LDraw files
  - Handles color definitions, document structures, element types
  - Provides async library loading and part resolution
  - Key modules: `color`, `document`, `elements`, `parser`, `library`

- **`ldraw-ir`**: Intermediate representation for 3D geometry
  - Converts LDraw elements to vertex/index buffers
  - Manages mesh grouping by material properties
  - Uses KD-trees for efficient vertex deduplication
  - Key modules: `geometry`, `model`, `part`

- **`ldraw-renderer`**: GPU rendering using wgpu
  - Manages render pipelines and GPU resources
  - Supports instanced rendering and object selection
  - Handles opaque/translucent sorting
  - Key modules: `pipeline`, `display_list`, `part`, `projection`

- **`ldraw-olr`**: Offline/headless rendering
  - Provides rendering without window context
  - Used by command-line tools for image generation

### Key Concepts

- **Part Alias**: Normalized part names (case-insensitive, path separator agnostic)
- **BFC (Back Face Culling)**: Winding order management for optimized rendering
- **Mesh Groups**: Geometry grouped by color and BFC state
- **Entity System**: GPU state synchronization with update tracking

### Platform Support

- Native targets: Full async runtime with tokio
- WebAssembly: Uses wasm-bindgen with appropriate web APIs
- Different HTTP clients for native (reqwest) vs WASM (gloo)

## Development Workflow

When modifying the codebase:

1. Follow existing code patterns and style
2. Use the async patterns for I/O operations
3. Maintain strong typing with custom types (ColorReference, PartAlias, etc.)
4. Ensure proper error handling throughout
5. Test both native and WASM targets when applicable

For GPU-related changes:
- Check wgpu 25.0 compatibility
- Use bytemuck for zero-copy buffer serialization
- Follow the entity update pattern for GPU state changes

When working with LDraw files:
- Respect BFC certification and winding orders
- Handle multipart documents correctly
- Maintain part alias normalization