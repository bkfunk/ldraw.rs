# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LDraw.rs is a Rust library for manipulating and rendering LDraw (virtual LEGO CAD) model files. It can be compiled to WebAssembly for browser-based rendering.

## Current Project

The goal of the current development project, which uses a fork of LDraw.rs, is to allow other programs, and in particular an app built in Godot for designing and simulating Lego creations, to import, process, and manipulate parts and models from their corresponding LDraw files. In particular, the workflow is as follows:

- Use existing functionality of LDraw.rs to create Rust objects for all the LDraw parts in a particular folder
- Add any additional data required to render and manipulate in a CAD-type program
- Supplement those Rust objects with metadata about how each part can connect to other parts (e.g. where the studs are, or, for a technic pin, say, how much rotational friction there is when connected to a pin hole).
- Create a Rust index of all the parts by various attributes (name, size, available connection types, etc.) to allow for fast part lookup and search.
- Once all parts are represented fully in Rust data types, we can then do any of:
  1. Export those data types to various file formats (e.g. stl, obj)
  2. Use those data types directly in Godot using the [godot-rust](https://godot-rust.github.io/docs/gdext/master/godot/) project, which provides Rust bindings for Godot 4.
  3. Manipulate them in native Rust code

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

## Current Development Status

### Connection System
The project has implemented a foundation for part connections:
- **Coupling Types**: Comprehensive definitions for LEGO connections (studs, pins, axles, clips, hinges)
- **Connection Properties**: Physical properties including friction, break forces, and movement constraints
- **Connection Graph**: System for tracking active connections between parts
- **Coupling Patterns**: Helper functions for generating common LEGO coupling configurations

### High-Level Development Plan

#### Phase 1: Coupling Detection and Integration
1. **Complete ldraw_converter tool** - Process LDraw files and automatically detect coupling points from primitives
2. **Implement coupling detection algorithm** - Analyze standard LDraw primitives (stud.dat, etc.) to identify connection locations
3. **Extend Part structure** - Add coupling metadata to the existing Part representation

#### Phase 2: Indexing and Search
1. **Build part index system** - Create searchable data structures indexed by name, dimensions, coupling types
2. **Implement query API** - Enable finding parts by various attributes and compatible connections
3. **Add categorization** - Automatically categorize parts based on type and function

#### Phase 3: Export and Integration
1. **Add export functionality** - Support STL, OBJ, and other 3D formats
2. **Prepare Godot integration** - Create compatible data structures for godot-rust bindings
3. **Optimize for real-time use** - Ensure performance meets CAD application requirements

### Critical Path Forward

1. **First Priority**: Complete ldraw_converter implementation
   - Parse LDraw files and detect coupling metadata from primitives/subparts
   - Store coupling information alongside geometry data

2. **Second Priority**: Automatic coupling detection
   - Map primitive usage (stud.dat, pin.dat, etc.) to coupling locations
   - Handle transformations to get correct world-space positions

3. **Third Priority**: Part indexing system
   - Enable fast lookup by name, size, and connection compatibility
   - Support complex queries for part selection

4. **Fourth Priority**: Basic export functionality
   - Start with STL format as it's widely supported
   - Ensure coupling metadata is preserved or exported separately

5. **Final Priority**: Godot preparation
   - Structure data for efficient use in game engine
   - Consider serialization format for cross-language compatibility