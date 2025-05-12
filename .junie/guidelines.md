# Filer Project Development Guidelines

This document provides essential information for developers working on the Filer project.

## Build and Configuration Instructions

### Prerequisites
- Rust toolchain (latest stable version recommended)
- Node.js and pnpm for frontend development

### Building the Project
1. **Clone the repository**:
   ```bash
   git clone https://github.com/Drew-Chase/filer.git
   cd filer
   ```

2. **Build the Rust backend**:
   ```bash
   cargo build
   ```

3. **Build the frontend**:
   ```bash
   pnpm install
   pnpm build
   ```

4. **Run the development server**:
   ```bash
   cargo run
   ```
   This will start both the Rust backend server and the Vite development server for the frontend.

### Configuration
- The server runs on port 7667 by default (defined in `src-actix/helpers/constants.rs`)
- Debug mode is automatically enabled in debug builds using `cfg!(debug_assertions)`
- The project creates two directories during build:
  - `target/dev-env`: For development environment files
  - `target/wwwroot`: For static web files

## Testing Information

### Running Tests
To run all tests in the project:
```bash
cargo test
```

To run tests for a specific module:
```bash
cargo test <module_name>
```

For example, to run only the filesystem tests:
```bash
cargo test filesystem
```

### Adding New Tests
1. Create a test file in the appropriate module directory (e.g., `src-actix/module_name/module_test.rs`)
2. Add the test module to the module's `mod.rs` file:
   ```rust
   #[cfg(test)]
   mod module_test;
   ```
3. Write your tests using the standard Rust testing framework:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       // Import any other necessary modules

       #[test]
       fn test_something() {
           // Test code here
           assert_eq!(expected, actual);
       }
   }
   ```

### Test Example
Here's a simple test for the `FilesystemEntry` struct:

```rust
#[cfg(test)]
mod tests {
    use crate::filesystem::filesystem_data::FilesystemEntry;
    use std::time::SystemTime;

    #[test]
    fn test_filesystem_entry_creation() {
        let now = SystemTime::now();
        let entry = FilesystemEntry {
            filename: "test_file.txt".to_string(),
            path: "/test/path".to_string(),
            is_dir: false,
            size: 1024,
            created: Some(now),
            last_modified: Some(now),
        };

        assert_eq!(entry.filename, "test_file.txt");
        assert_eq!(entry.path, "/test/path");
        assert_eq!(entry.is_dir, false);
        assert_eq!(entry.size, 1024);
        assert_eq!(entry.created, Some(now));
        assert_eq!(entry.last_modified, Some(now));
    }
}
```

## Project Structure

### Backend (Rust)
- **src-actix/**: Contains the Rust backend code
  - **main.rs**: Entry point for the application
  - **lib.rs**: Main library code and server setup
  - **auth/**: Authentication-related code
  - **filesystem/**: Filesystem operations
  - **helpers/**: Utility functions and constants

### Frontend
- **src/**: Contains the frontend code
  - Uses TypeScript, React, and Tailwind CSS
  - Built with Vite

## Development Guidelines

### Code Style
- Follow standard Rust coding conventions
- Use `cargo fmt` to format code
- Use `cargo clippy` for linting

### Error Handling
- Use the `anyhow` crate for general error handling
- Use `thiserror` for defining custom error types
- Return `Result<T, Error>` from functions that can fail

### API Development
- API endpoints are defined in the respective module's `*_endpoint.rs` files
- Use the Actix web framework for defining routes and handlers
- API routes are mounted under the `/api` path

### Database
- The project uses SQLite with the `sqlx` crate
- Database initialization happens in `auth_db::initialize()`

### Authentication
- Authentication is handled by the `auth` module
- User permissions are defined using flags in `permission_flags.rs`

## Debugging

### Logs
- The application uses the `log` crate with `pretty_env_logger`
- Log level is set to `Debug` by default
- In development mode, both backend and frontend logs are displayed

### Development Mode
- Development mode is automatically enabled in debug builds
- It starts a Vite development server for hot-reloading the frontend
- The backend API is available at `http://localhost:7667/api`